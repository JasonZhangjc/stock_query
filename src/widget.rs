use tui::{layout::{Rect, Layout, Direction, Constraint, Alignment}, 
widgets::{Paragraph, Block, Borders, BorderType, List, ListItem}, 
style::{Style, Color, Modifier}, text::{Spans, Span}};

use crate::{App, Stock, AppState};
use unicode_width::UnicodeWidthStr;


const VERSION:&str = env!("CARGO_PKG_VERSION");


// calculate the area of the screen window, in order for being used later to render
// Rect: A simple rectangle used in the computation of the layout and to give widgets an hint about the area they are supposed to render to.
// area is a rectangle shape
// returns a vector of rectangles
// TUI for the App window
pub fn main_chunks(area: Rect) -> Vec<Rect> {
    let parent = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ].as_ref())
        .split(area);

    // center position
    let center = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ].as_ref())
        .split(parent[1]);

    // calculate the popup window when adding a new stock    
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(40),
        ].as_ref())
        .split(area); 
    let popline = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ].as_ref())
        .split(popup[1]);       
    
    vec!(parent[0], center[0], center[1], parent[2], popline[1])    
}

// TUI for stock list
pub fn stock_list(stocks: &Vec<Stock>) -> List {
    let items: Vec<_> = stocks.iter()
        .map(|stock| {
            ListItem::new(Spans::from(vec![
                Span::styled(format!("{:+.2}% ",stock.percent * 100.0),
                    Style::default().fg(if stock.percent < 0.0 {Color::Green} else {Color::Red})),
                Span::styled(stock.title.clone(),Style::default()),
                ]))
        }).collect();

    List::new(items)
        .block(
            Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("LIST")
            .border_type(BorderType::Plain))
        .highlight_style(
            Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD))
}

// TUI for stock detail
pub fn stock_detail(app: &App) -> Paragraph {
    let mut info = String::new();
    let sel = app.stocks_state.selected().unwrap_or(0);
    // prevent sel from exceeding the list range
    let stocks = app.stocks.lock().unwrap();
    if app.stocks_state.selected().is_some() && sel < stocks.len() {
        let stock = stocks.get(sel).unwrap();
        info = format!("CODE:{}\nUP_DOWN:{:+.2}%\nCURRENT:{}\nOPEN:{}\nYESTERDAY_CLOSE:{}\nHIGH:{}\nLOW:{}", 
            stock.code, stock.percent * 100.0, stock.price, stock.open, stock.yestclose, stock.high, stock.low);
    }

    Paragraph::new(info)
        .alignment(Alignment::Center)
        .style(Style::default())
        .block(Block::default().title("DETAIL")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain))
}

pub fn stock_input(app: &App) -> Paragraph {
    Paragraph::new(app.input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("ENDER STOCK CODE"))
}

pub fn title_bar(app: &App, rect: Rect) -> Paragraph {
    let left = format!("Stock v{}", VERSION);
    let error = app.error.lock().unwrap();
    let right = if error.is_empty() { app.last_refresh.lock().unwrap().format("LAST UPDATE %H:%M:%S").to_string() } else { error.clone() };
    Paragraph::new(Spans::from(vec![
        Span::raw(left.clone()),
        // Use checked_sub to prevent overflow
        Span::raw(" ".repeat((rect.width as usize).checked_sub(right.width() + left.width()).unwrap_or(0))),
        Span::styled(right,Style::default()
            .fg(if error.is_empty() { Color::White } else { Color::Red })),
        ]))
    .alignment(Alignment::Left)
}

// Status bar
pub fn status_bar(app: &mut App) -> Paragraph {    
    Paragraph::new(match app.state {
            // at Normal AppState when reading stocks
            AppState::Normal => "EXIT[Q] | NEW[N] | DEL[D] | REFRESH[R] | UP[U] | DOWN[J]", 
            // at Adding AppState when adding stocks
            AppState::Adding => "ENTER[Enter] | CANCELL[ESC] | ADD 0 AHEAD OF SHANGHAI STOCK EXCHANGE CODE, 1 FOR SHENZHEN STOCK EXCHANGE"
        }.to_string()
    ).alignment(Alignment::Left)
}
