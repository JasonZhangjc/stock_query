// import three different enum types
// use keyboard code and mouse events
use crossterm::event::{KeyCode, Event, MouseEventKind};

use crate::{App, AppState, Stock};

// handle keyboard and mouse events
pub fn on_events(event:Event, app:&mut App) {
    let total = app.stocks.lock().unwrap().len(); 
    // to indicate which stock is selected
    let sel = app.stocks_state.selected().unwrap_or(0);
    // to indicate whether some stock is selected
    let selsome = app.stocks_state.selected().is_some() && sel < total;
    match app.state {
        // Normal AppState
        AppState::Normal => {
            // Keyboard events ---------------------------------------------------------------------------------
            if let Event::Key(key) = event {
                let code = key.code;
                // Use 'q' and 'Q' to exit the TUI
                if code == KeyCode::Char('q') || code == KeyCode::Char('Q') {
                    app.should_exit = true;
                }
                // Use 'r' and 'R' to refresh stock panel
                else if code == KeyCode::Char('r') || code == KeyCode::Char('R') { 
                    app.refresh_stocks();
                }
                // Use 'n' and 'N' to add new stock
                else if code == KeyCode::Char('n') || code == KeyCode::Char('N') {
                    // add a stock
                    app.state = AppState::Adding;
                    app.input = String::new();
                }
                // if some stock is selected
                // Use 'd' and 'D' to delete a selected stock
                else if (code == KeyCode::Char('d') || code == KeyCode::Char('D')) && selsome {
                    // delete the selected stock
                    app.stocks.lock().unwrap().remove(sel);
                    app.save_stocks().unwrap();
                    app.stocks_state.select(None);
                }
                // if some stock is selected and the selected is not at the top of the panel
                // Use 'u' and 'U' to move the selected stock upward
                else if (code == KeyCode::Char('u') || code == KeyCode::Char('U')) && selsome && sel > 0 {
                    // move upward
                    app.stocks.lock().unwrap().swap(sel, sel -1);
                    app.save_stocks().unwrap();
                    app.stocks_state.select(Some(sel - 1));
                }
                // if some stock is selected and the selected is not at the bottom of the panel
                // Use 'j' and 'J' to move the selected stock downward
                else if (code == KeyCode::Char('j') || code == KeyCode::Char('J')) && selsome && sel < total - 1 {
                    // move downward
                    app.stocks.lock().unwrap().swap(sel, sel + 1);
                    app.save_stocks().unwrap();
                    app.stocks_state.select(Some(sel + 1));
                }
                // if we want to move upward and there are stocks on the panel
                // Use 'up' on the keyboard to move upward
                else if code == KeyCode::Up && total > 0 {
                    // need to evaluate sel>0 to avoid exception
                    app.stocks_state.select(Some(if sel > 0 {sel - 1} else {0}));
                }
                // if we want to move downward and there are stocks on the panel
                // Use 'down' on the keyboard to move downward
                else if code == KeyCode::Down && total > 0 {
                    // need to evaluate sel<total-1 to avoid exception
                    app.stocks_state.select(Some(if sel < total - 1 {sel + 1} else {sel}));
                }
            }
            // Mouse events -----------------------------------------------------------------------------------
            else if let Event::Mouse(mouse) = event {
                // move upward via mouse
                if let MouseEventKind::Up(_button) = mouse.kind {
                    let row = mouse.row as usize; 
                    // list starts from line 3
                    // thus minus 2
                    if row >= 2 && row < total + 2{
                        app.stocks_state.select(Some(row - 2));
                    }
                }
            }
        },

        // Adding AppState
        AppState::Adding => match event {
            Event::Key(key) => match key.code {
                // Use 'Enter' on the keyboard to add a new stock via inputs
                KeyCode::Enter => {
                    app.state = AppState::Normal;
                    if app.input.len() > 0 {
                        app.stocks.lock().unwrap().push(Stock::new(&app.input));
                        app.refresh_stocks();
                        app.save_stocks().unwrap();
                    }
                }
                // Use 'Esc' on the keyboard to exit from the Adding AppState and enter the Normal AppState
                KeyCode::Esc => {
                    app.state = AppState::Normal;
                }
                // Use 'any other characters' on the keyboard to push chars into the input
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                // Use 'Backspace' on the keyboard to pop the last char from the input
                KeyCode::Backspace => {
                    app.input.pop();
                }
                // any other, do nothing
                _ => {}
            },
            // any other, do nothing
            _ => {},
        }
    }
}

// handle timing event
pub fn on_tick(app:&mut App) {
    app.tick_count+=1;
    // Every 1 min, refresh stocks
    if app.tick_count % 60 == 0  {
        if  let AppState::Normal = app.state {  
            app.refresh_stocks();
        }
    }
}
