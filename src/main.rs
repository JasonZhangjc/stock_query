use std::{error::Error, time::{Instant, Duration}};

use stock::{DynResult, CrossTerminal, App, TerminalFrame, events, widget, AppState};
use tui::{Terminal, backend::CrosstermBackend, widgets};
use unicode_width::UnicodeWidthStr;


// TUI

fn main() -> DynResult{
    let mut app = App::new();
    let mut terminal = init_terminal()?;
    // main_loop contains majority of functionality
    main_loop(&mut terminal, &mut app)?;
    close_terminal(terminal)?;

    Ok(())
}

fn init_terminal() -> Result<CrossTerminal, Box<dyn Error>> {
    let mut stdout = std::io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    // Excute EnableMouseCapture to support Mouse events
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn close_terminal(mut terminal: CrossTerminal) -> DynResult{
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::event::DisableMouseCapture, crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

// main loop for most events
fn main_loop(terminal: &mut CrossTerminal, app: &mut App) -> DynResult {
    let mut last_tick = Instant::now();
    while !app.should_exit {
        terminal.draw(|f| {on_draw(f, app);})?;

        if crossterm::event::poll(Duration::from_secs(1).checked_sub(last_tick.elapsed()).unwrap_or_default())? {
            events::on_events(crossterm::event::read()?, app);
        }
        else {
            events::on_tick(app);
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn on_draw(frame: &mut TerminalFrame, app: &mut App) {
    let chunks = widget::main_chunks(frame.size());
    
    // need to tune render_stateful_widget when rendering the list
    // otherwise the rolling status is incorrect,
    // the first parameter cannot be 'app',
    // otherwise it conflicts with 'mut stock_state'
    frame.render_stateful_widget(widget::stock_list(&app.stocks.lock().unwrap()), chunks[1], &mut app.stocks_state);
    
    // Since rendering stock list would change the rolling status, 
    // if this value is needed later, has to do the list rendering
    frame.render_widget(widget::title_bar(app, frame.size()), chunks[0]);
    frame.render_widget(widget::stock_detail(app), chunks[2]);
    frame.render_widget(widget::status_bar(app), chunks[3]);

    if let AppState::Adding = app.state {
        // clear before popup, otherwise the background color would also popup
        frame.render_widget(widgets::Clear, chunks[4]);
        frame.render_widget(widget::stock_input(app), chunks[4]);
        
        // display the cursor
        // width() interface depends on an external lib
        // can handle text width for many languages including Mandarin
        frame.set_cursor(chunks[4].x + app.input.width() as u16 + 1, chunks[4].y + 1);
    }
    
}