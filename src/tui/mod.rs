use std::io;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use image::DynamicImage;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect, Size};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::Protocol;
use ratatui_image::{Image, Resize};
use tokio::runtime::Handle;

use crate::app::{SessionConfig, SessionStarted, build_report, start_server_session};

const DEFAULT_PORT: &str = "8787";
const SPINNER_FRAMES: [&str; 8] = ["-", "\\", "|", "/", "-", "\\", "|", "/"];

pub fn run(runtime: Handle) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = TuiApp::new()?;
    let result = run_loop(&mut terminal, &mut app, runtime);
    restore_terminal(&mut terminal)?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    runtime: Handle,
) -> io::Result<()> {
    loop {
        if let Screen::Loading = app.screen {
            app.poll_worker();
        }

        terminal.draw(|frame| app.render(frame))?;

        if app.cursor_position.is_some() {
            terminal.show_cursor()?;
        } else {
            terminal.hide_cursor()?;
        }

        if app.should_exit {
            return Ok(());
        }

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key.code, runtime.clone());
        }

        app.tick();
    }
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Splash,
    Form,
    Loading,
    Result,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormField {
    Input,
    Port,
    Open,
    Start,
}

impl FormField {
    fn next(self) -> Self {
        match self {
            Self::Input => Self::Port,
            Self::Port => Self::Open,
            Self::Open => Self::Start,
            Self::Start => Self::Input,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Input => Self::Start,
            Self::Port => Self::Input,
            Self::Open => Self::Port,
            Self::Start => Self::Open,
        }
    }
}

enum WorkerMessage {
    Status(String),
    Finished(Result<SessionStarted, String>),
}

struct TuiApp {
    screen: Screen,
    field: FormField,
    input: String,
    input_cursor: usize,
    port: String,
    port_cursor: usize,
    open_browser: bool,
    splash_image: DynamicImage,
    splash_picker: Picker,
    splash_protocol: Option<Protocol>,
    splash_protocol_size: Option<Size>,
    splash_error: Option<String>,
    status_lines: Vec<String>,
    spinner_index: usize,
    worker_rx: Option<Receiver<WorkerMessage>>,
    session: Option<SessionStarted>,
    error_message: Option<String>,
    cursor_position: Option<(u16, u16)>,
    should_exit: bool,
    last_tick: Instant,
}

impl TuiApp {
    fn new() -> io::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let splash_image = image::load_from_memory(include_bytes!("../assets/atlas.png"))
            .map_err(io::Error::other)?;
        let splash_picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());

        Ok(Self {
            screen: Screen::Splash,
            field: FormField::Input,
            input: current_dir.display().to_string(),
            input_cursor: current_dir.display().to_string().chars().count(),
            port: DEFAULT_PORT.to_string(),
            port_cursor: DEFAULT_PORT.chars().count(),
            open_browser: true,
            splash_image,
            splash_picker,
            splash_protocol: None,
            splash_protocol_size: None,
            splash_error: None,
            status_lines: vec!["Aguardando inicio do mapeamento...".to_string()],
            spinner_index: 0,
            worker_rx: None,
            session: None,
            error_message: None,
            cursor_position: None,
            should_exit: false,
            last_tick: Instant::now(),
        })
    }

    fn tick(&mut self) {
        if self.last_tick.elapsed() >= Duration::from_millis(120) {
            self.spinner_index = (self.spinner_index + 1) % SPINNER_FRAMES.len();
            self.last_tick = Instant::now();
        }
    }

    fn poll_worker(&mut self) {
        if let Some(rx) = &self.worker_rx {
            while let Ok(message) = rx.try_recv() {
                match message {
                    WorkerMessage::Status(status) => self.status_lines.push(status),
                    WorkerMessage::Finished(result) => {
                        self.worker_rx = None;
                        match result {
                            Ok(session) => {
                                self.session = Some(session);
                                self.screen = Screen::Result;
                            }
                            Err(error) => {
                                self.error_message = Some(error);
                                self.screen = Screen::Error;
                            }
                        }
                        break;
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode, runtime: Handle) {
        match self.screen {
            Screen::Splash => match code {
                KeyCode::Enter => self.screen = Screen::Form,
                KeyCode::Esc | KeyCode::Char('q') => self.should_exit = true,
                _ => {}
            },
            Screen::Form => self.handle_form_key(code, runtime),
            Screen::Loading => {
                if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
                    self.should_exit = true;
                }
            }
            Screen::Result => match code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
                KeyCode::Char('o') => {
                    if let Some(session) = &self.session
                        && let Err(error) = open::that(&session.local_url)
                    {
                        self.error_message = Some(format!(
                            "falha ao abrir o navegador automaticamente: {error}"
                        ));
                        self.screen = Screen::Error;
                    }
                }
                KeyCode::Char('n') => {
                    self.session = None;
                    self.error_message = None;
                    self.status_lines = vec!["Aguardando inicio do mapeamento...".to_string()];
                    self.input_cursor = self.input.chars().count();
                    self.port_cursor = self.port.chars().count();
                    self.screen = Screen::Form;
                }
                _ => {}
            },
            Screen::Error => match code {
                KeyCode::Esc | KeyCode::Char('q') => self.should_exit = true,
                KeyCode::Enter => {
                    self.error_message = None;
                    self.status_lines = vec!["Aguardando inicio do mapeamento...".to_string()];
                    self.screen = Screen::Form;
                }
                _ => {}
            },
        }
    }

    fn handle_form_key(&mut self, code: KeyCode, runtime: Handle) {
        match code {
            KeyCode::Tab | KeyCode::Down => self.field = self.field.next(),
            KeyCode::BackTab | KeyCode::Up => self.field = self.field.previous(),
            KeyCode::Left => match self.field {
                FormField::Input => self.input_cursor = self.input_cursor.saturating_sub(1),
                FormField::Port => self.port_cursor = self.port_cursor.saturating_sub(1),
                _ => {}
            },
            KeyCode::Right => match self.field {
                FormField::Input => {
                    self.input_cursor = (self.input_cursor + 1).min(self.input.chars().count())
                }
                FormField::Port => {
                    self.port_cursor = (self.port_cursor + 1).min(self.port.chars().count())
                }
                _ => {}
            },
            KeyCode::Home => match self.field {
                FormField::Input => self.input_cursor = 0,
                FormField::Port => self.port_cursor = 0,
                _ => {}
            },
            KeyCode::End => match self.field {
                FormField::Input => self.input_cursor = self.input.chars().count(),
                FormField::Port => self.port_cursor = self.port.chars().count(),
                _ => {}
            },
            KeyCode::Enter => {
                if self.field == FormField::Start {
                    self.start_loading(runtime);
                } else if self.field == FormField::Open {
                    self.open_browser = !self.open_browser;
                } else {
                    self.field = self.field.next();
                }
            }
            KeyCode::Char(' ') if self.field == FormField::Open => {
                self.open_browser = !self.open_browser;
            }
            KeyCode::Backspace => match self.field {
                FormField::Input => {
                    if self.input_cursor > 0 {
                        remove_char_at(&mut self.input, self.input_cursor - 1);
                        self.input_cursor -= 1;
                    }
                }
                FormField::Port => {
                    if self.port_cursor > 0 {
                        remove_char_at(&mut self.port, self.port_cursor - 1);
                        self.port_cursor -= 1;
                    }
                }
                _ => {}
            },
            KeyCode::Delete => match self.field {
                FormField::Input => {
                    if self.input_cursor < self.input.chars().count() {
                        remove_char_at(&mut self.input, self.input_cursor);
                    }
                }
                FormField::Port => {
                    if self.port_cursor < self.port.chars().count() {
                        remove_char_at(&mut self.port, self.port_cursor);
                    }
                }
                _ => {}
            },
            KeyCode::Char(character) => match self.field {
                FormField::Input => {
                    insert_char_at(&mut self.input, self.input_cursor, character);
                    self.input_cursor += 1;
                }
                FormField::Port if character.is_ascii_digit() => {
                    insert_char_at(&mut self.port, self.port_cursor, character);
                    self.port_cursor += 1;
                }
                _ => {}
            },
            KeyCode::Esc => self.should_exit = true,
            _ => {}
        }
    }

    fn start_loading(&mut self, runtime: Handle) {
        let port = match self.port.parse::<u16>() {
            Ok(port) => port,
            Err(_) => {
                self.error_message = Some("porta invalida. informe um numero entre 1 e 65535".to_string());
                self.screen = Screen::Error;
                return;
            }
        };

        let config = SessionConfig {
            input: self.input.trim().into(),
            port,
            open_browser: self.open_browser,
        };

        let (tx, rx) = mpsc::channel();
        self.worker_rx = Some(rx);
        self.status_lines = vec!["Iniciando fluxo do Atlas...".to_string()];
        self.screen = Screen::Loading;

        std::thread::spawn(move || {
            let _ = tx.send(WorkerMessage::Status("Validando diretorio informado...".to_string()));

            match build_report(&config) {
                Ok(report) => {
                    let _ = tx.send(WorkerMessage::Status("Mapeamento concluido. Iniciando live server...".to_string()));
                    let result = runtime.block_on(start_server_session(report, config.port, config.open_browser));
                    let _ = tx.send(WorkerMessage::Finished(result));
                }
                Err(error) => {
                    let _ = tx.send(WorkerMessage::Finished(Err(error)));
                }
            }
        });
    }

    fn render(&mut self, frame: &mut ratatui::Frame) {
        self.cursor_position = None;

        match self.screen {
            Screen::Splash => self.render_splash(frame),
            Screen::Form => self.render_form(frame),
            Screen::Loading => self.render_loading(frame),
            Screen::Result => self.render_result(frame),
            Screen::Error => self.render_error(frame),
        }
    }

    fn render_splash(&mut self, frame: &mut ratatui::Frame) {
        let area = centered_rect(frame.area(), 94, 94);
        frame.render_widget(Clear, area);
        let block = Block::default().title("Atlas").borders(Borders::ALL);
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(12),
                Constraint::Length(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new("Atlas carrega o acervo e entrega o live server local em um fluxo guiado.")
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            sections[0],
        );

        let image_area = sections[1];
        let image_size = Size::new(image_area.width, image_area.height);

        if image_area.width > 0
            && image_area.height > 0
            && self.splash_protocol_size != Some(image_size)
        {
            match self.splash_picker.new_protocol(
                self.splash_image.clone(),
                image_size,
                Resize::Fit(None),
            ) {
                Ok(protocol) => {
                    self.splash_protocol = Some(protocol);
                    self.splash_protocol_size = Some(image_size);
                    self.splash_error = None;
                }
                Err(error) => {
                    self.splash_protocol = None;
                    self.splash_protocol_size = None;
                    self.splash_error = Some(error.to_string());
                }
            }
        }

        if let Some(protocol) = &self.splash_protocol {
            let protocol_size = protocol.size();
            let centered_image_area = centered_area_for_size(image_area, protocol_size.width, protocol_size.height);
            frame.render_widget(Image::new(protocol), centered_image_area);
        } else {
            let fallback = self
                .splash_error
                .as_deref()
                .unwrap_or("Nao foi possivel renderizar a imagem do Atlas neste terminal.");
            frame.render_widget(
                Paragraph::new(fallback)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true }),
                image_area,
            );
        }

        frame.render_widget(
            Paragraph::new(Text::from(Line::from(Span::styled(
                "Pressione Enter para continuar",
                Style::default().add_modifier(Modifier::BOLD),
            ))))
            .alignment(Alignment::Center),
            sections[3],
        );
        frame.render_widget(
            Paragraph::new("Esc ou Q para sair").alignment(Alignment::Center),
            sections[4],
        );
    }

    fn render_form(&mut self, frame: &mut ratatui::Frame) {
        let area = centered_rect(frame.area(), 82, 88);
        frame.render_widget(Clear, area);
        let block = Block::default().title("Atlas | Novo mapeamento").borders(Borders::ALL);
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new("Configure o diretorio de origem e inicie o live server sem depender de argumentos de terminal.")
                .wrap(Wrap { trim: true }),
            sections[0],
        );

        let input_focused = self.field == FormField::Input;
        frame.render_widget(
            input_block(
                "Diretorio",
                &self.input,
                input_focused,
                "Digite o caminho completo do diretorio a ser analisado.",
            ),
            sections[1],
        );
        if input_focused {
            self.cursor_position = Some(input_cursor_position(sections[1], self.input_cursor));
        }

        let port_focused = self.field == FormField::Port;
        frame.render_widget(
            input_block(
                "Porta",
                &self.port,
                port_focused,
                "A porta padrao do Atlas e 8787.",
            ),
            sections[2],
        );
        if port_focused {
            self.cursor_position = Some(input_cursor_position(sections[2], self.port_cursor));
        }

        let toggle = if self.open_browser { "[x] Abrir navegador automaticamente" } else { "[ ] Abrir navegador automaticamente" };
        frame.render_widget(
            Paragraph::new(toggle).block(focus_block("Comportamento", self.field == FormField::Open)),
            sections[3],
        );
        frame.render_widget(
            Paragraph::new("Iniciar mapeamento")
                .alignment(Alignment::Center)
                .style(if self.field == FormField::Start {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .block(focus_block("Acao", self.field == FormField::Start)),
            sections[4],
        );
        frame.render_widget(
            Paragraph::new("Tab/Shift+Tab navegam | Setas movem o cursor | Enter confirma | Espaco alterna a abertura do navegador | Esc sai")
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true }),
            sections[5],
        );
    }

    fn render_loading(&self, frame: &mut ratatui::Frame) {
        let area = centered_rect(frame.area(), 76, 70);
        frame.render_widget(Clear, area);
        let block = Block::default().title("Atlas | Processando").borders(Borders::ALL);
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
            .split(inner);

        let spinner = format!(
            "{} Executando mapeamento e subida do live server...",
            SPINNER_FRAMES[self.spinner_index]
        );
        frame.render_widget(Paragraph::new(spinner).alignment(Alignment::Center), sections[0]);

        let status_lines = self
            .status_lines
            .iter()
            .rev()
            .take(12)
            .rev()
            .cloned()
            .map(Line::from)
            .collect::<Vec<_>>();
        frame.render_widget(
            Paragraph::new(Text::from(status_lines))
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .wrap(Wrap { trim: true }),
            sections[1],
        );
        frame.render_widget(
            Paragraph::new("Pressione Q para sair a qualquer momento.").alignment(Alignment::Center),
            sections[2],
        );
    }

    fn render_result(&self, frame: &mut ratatui::Frame) {
        let Some(session) = &self.session else { return; };
        let area = centered_rect(frame.area(), 84, 86);
        frame.render_widget(Clear, area);
        let block = Block::default().title("Atlas | Sessao iniciada").borders(Borders::ALL);
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(6),
                Constraint::Length(5),
                Constraint::Min(4),
                Constraint::Length(2),
            ])
            .split(inner);

        frame.render_widget(
            Paragraph::new(format!(
                "Servidor local iniciado com sucesso.\nURL: {}",
                session.local_url
            ))
            .block(Block::default().borders(Borders::ALL).title("Live server"))
            .wrap(Wrap { trim: true }),
            sections[0],
        );

        frame.render_widget(
            Paragraph::new(format!(
                "Origem: {}\nGerado em: {}\nAbrir navegador: {}",
                session.report.source,
                session.report.generated_at,
                if session.browser_error.is_some() { "falhou" } else if self.open_browser { "sim" } else { "nao" }
            ))
            .block(Block::default().borders(Borders::ALL).title("Sessao"))
            .wrap(Wrap { trim: true }),
            sections[1],
        );

        frame.render_widget(
            Paragraph::new(format!(
                "Diretorios: {}\nArquivos: {}\nItens ignorados: {}\nAvisos: {}",
                session.report.summary.total_directories,
                session.report.summary.total_files,
                session.report.summary.ignored_items,
                session.report.summary.warning_count,
            ))
            .block(Block::default().borders(Borders::ALL).title("Resumo")),
            sections[2],
        );

        let mut warnings = if session.report.warnings.is_empty() {
            vec![Line::from("Nenhum aviso registrado durante o mapeamento.")]
        } else {
            session
                .report
                .warnings
                .iter()
                .take(8)
                .map(|warning| {
                    Line::from(format!(
                        "{} | {} | {}",
                        warning.kind, warning.path, warning.message
                    ))
                })
                .collect::<Vec<_>>()
        };
        if let Some(browser_error) = &session.browser_error {
            warnings.push(Line::from(format!("Falha ao abrir navegador: {browser_error}")));
        }

        frame.render_widget(
            Paragraph::new(Text::from(warnings))
                .block(Block::default().borders(Borders::ALL).title("Avisos"))
                .wrap(Wrap { trim: true }),
            sections[3],
        );

        frame.render_widget(
            Paragraph::new("O abre o navegador | N inicia novo mapeamento | Q sai")
                .alignment(Alignment::Center),
            sections[4],
        );
    }

    fn render_error(&self, frame: &mut ratatui::Frame) {
        let area = centered_rect(frame.area(), 70, 40);
        frame.render_widget(Clear, area);
        let block = Block::default().title("Atlas | Erro").borders(Borders::ALL);
        frame.render_widget(block.clone(), area);
        let inner = block.inner(area);
        let message = self
            .error_message
            .as_deref()
            .unwrap_or("Ocorreu um erro inesperado.");
        frame.render_widget(
            Paragraph::new(format!("{}\n\nPressione Enter para voltar ou Q para sair.", message))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true }),
            inner,
        );
    }
}

fn focus_block<T>(title: T, focused: bool) -> Block<'static>
where
    T: Into<Line<'static>>,
{
    let style = if focused {
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    Block::default().title(title).borders(Borders::ALL).border_style(style)
}

fn input_block<'a>(title: &'a str, value: &'a str, focused: bool, help: &'a str) -> Paragraph<'a> {
    let title = if focused {
        format!("> {title}")
    } else {
        title.to_string()
    };

    Paragraph::new(format!("{}\n{}", value, help))
        .block(focus_block(title, focused))
        .wrap(Wrap { trim: true })
}

fn input_cursor_position(area: Rect, cursor_index: usize) -> (u16, u16) {
    let text_x = area.x.saturating_add(1);
    let text_y = area.y.saturating_add(1);
    let max_x = area.right().saturating_sub(2);
    let cursor_x = text_x
        .saturating_add(cursor_index as u16)
        .min(max_x.max(text_x));

    (cursor_x, text_y)
}

fn insert_char_at(value: &mut String, index: usize, character: char) {
    let byte_index = char_to_byte_index(value, index);
    value.insert(byte_index, character);
}

fn remove_char_at(value: &mut String, index: usize) {
    let start = char_to_byte_index(value, index);
    let end = char_to_byte_index(value, index + 1);
    value.replace_range(start..end, "");
}

fn char_to_byte_index(value: &str, char_index: usize) -> usize {
    value
        .char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(value.len())
}

fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn centered_area_for_size(area: Rect, width: u16, height: u16) -> Rect {
    let target_width = width.min(area.width).max(1);
    let target_height = height.min(area.height).max(1);
    let x = area.x + area.width.saturating_sub(target_width) / 2;
    let y = area.y + area.height.saturating_sub(target_height) / 2;

    Rect::new(x, y, target_width, target_height)
}
