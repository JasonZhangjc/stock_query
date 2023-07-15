/*
Structure:
        lib
        |
events, widget, aio
        |
        main

APP -> stock
*/

use std::{io::Stdout, fs, collections::HashMap, sync::{Mutex, Arc}, thread};

use chrono::{DateTime, Local};
use http_req::request;
use serde::{Serialize, Deserialize};
use serde_json::{Value, Map, json};
use tui::{backend::CrosstermBackend, widgets::ListState};

// can be visited outside this lib
pub mod events;
pub mod widget;
pub mod aio;

// Define types for convenience
// DynResult is a return type
// when ok, return nothing ()
// when not ok, return a dyn std error message
pub type DynResult = Result<(), Box<dyn std::error::Error>>;
// return type of tui terminal
pub type CrossTerminal = tui::Terminal<CrosstermBackend<Stdout>>;
pub type TerminalFrame<'a> = tui::Frame<'a, CrosstermBackend<Stdout>>;

pub const DB_PATH: &str=".stocks.json";

// Define stock as a struct
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stock {
    pub title: String,
    pub code: String,
    pub price: f64,
    pub percent: f64,
    pub open: f64,      // current open price
    pub yestclose: f64, // previous close price
    pub high: f64,      // current high price
    pub low: f64,       // current low price
}

impl Stock {
    // constructor
    pub fn new(code:&String) -> Self {
        Self {    // unmutable
            code: code.clone(),
            title: code.clone(),
            price:0.0,
            percent:0.0,
            open:0.0,
            yestclose:0.0,
            high:0.0,
            low:0.0,
        }
    }
}

// Define states of the APP as enum types
pub enum AppState {
    Normal,
    Adding,
}

// Define APP as a struct
pub struct App {
    pub should_exit:bool,
    pub state:AppState,
    pub error:Arc<Mutex<String>>,
    pub input:String,
    pub stocks:Arc<Mutex<Vec<Stock>>>,
    // ListState records the current selected position and the rolling position in the List module of TUI
    pub stocks_state:ListState,
    pub last_refresh:Arc<Mutex<DateTime<Local>>>,
    pub tick_count:u128,
}

impl App {
    // Constructor
    pub fn new() -> Self {
        let mut app = Self {   // mutable
            should_exit: false,
            state: AppState::Normal,
            input: String::new(),
            error: Arc::new(Mutex::new(String::new())),
            stocks: Arc::new(Mutex::new([].to_vec())),
            // ListState:default is 'unselected' as there might be no stocks
            stocks_state: ListState::default(),
            last_refresh: Arc::new(Mutex::new(Local::now())),
            tick_count: 0,
        };
        // load and refresh stocks
        app.load_stocks().unwrap_or_default();
        app.refresh_stocks();
        return app;
    }
    
    // save stocks info into a .json file
    pub fn save_stocks(&self) -> DynResult{
        let db=dirs_next::home_dir().unwrap().join(DB_PATH);
        // store each stock as an independent struct to allow future extendability.
        let stocks = self.stocks.lock().unwrap();
        let lists:Vec<_> = stocks.iter().map(|s| HashMap::from([("code", &s.code)])).collect();
        fs::write(&db, serde_json::to_string(&HashMap::from([("stocks", lists)]))?)?;
        Ok(())
    }

    // load stocks from a .json file
    pub fn load_stocks(&mut self) -> DynResult{
        // Use unwrap_or_default to avoid the exception when the file does not exist
        let content = fs::read_to_string(dirs_next::home_dir().unwrap().join(DB_PATH)).unwrap_or_default();
        // If we want to import stocks directly，the compatibility is bad as all keys have to be matched, as follows
        // self.stocks = serde_json::from_str(&content).unwrap_or_default();
        // Instead, first convert to Map to improve compatibility
        let json: Map<String, Value> = serde_json::from_str(&content).unwrap_or_default();
        let mut data = self.stocks.lock().unwrap();
        data.clear();
        // append stock in the stocks.json file
        // standard way of handling data
        data.append(&mut json.get("stocks").unwrap_or(&json!([])).as_array().unwrap().iter()
            .map(|s| Stock::new(&s.as_object().unwrap().get("code").unwrap().as_str().unwrap().to_string()))
            .collect());    
        // return ok
        Ok(())
    }

    pub fn refresh_stocks(&mut self) {
        let stock_clone = self.stocks.clone();
        let err_clone = self.error.clone();
        let last_refresh_clone = self.last_refresh.clone();
        // get codes of stocks
        let codes = self.get_codes();
        if codes.len() > 0 {
            thread::spawn(move || {
                let mut writer = Vec::new();
                // get stock data from online API
                let ret = request::get(format!("{}{}","http://api.money.126.net/data/feed/", codes), &mut writer);
                let mut locked_err = err_clone.lock().unwrap();
                if let Err(err) = ret {
                    *locked_err = format!("{:?}", err);
                }
                else {
                    let content = String::from_utf8_lossy(&writer);
                    if content.starts_with("_ntes_quote_callback") {
                        let mut stocks = stock_clone.lock().unwrap();  
                        // the data returned from the online API contains a javascript call
                        // we use skip,take,collect to realize a substring to extract info from that js call
                        let json: Map<String, Value> = serde_json::from_str(&content.chars().skip(21).take(content.len() - 23).collect::<String>()).unwrap();
                        for stock in stocks.iter_mut() {
                            // if the stock code is incorrect, then the returned json does not contain the info
                            // we should use unwrap_or to generatey something empty to avoid exception
                            let obj = json.get(&stock.code).unwrap_or(&json!({})).as_object().unwrap().to_owned();
                            stock.title = obj.get("name").unwrap_or(&json!(stock.code.clone())).as_str().unwrap().to_owned();
                            stock.price = obj.get("price").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.percent = obj.get("percent").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.open = obj.get("open").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.yestclose = obj.get("yestclose").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.high = obj.get("high").unwrap_or(&json!(0.0)).as_f64().unwrap();
                            stock.low = obj.get("low").unwrap_or(&json!(0.0)).as_f64().unwrap();

                            // if json.contains_key(&stock.code) {
                            //     let mut writer2 = Vec::new();
                            //     request::get(format!("http://img1.money.126.net/data/hs/time/today/{}.json",stock.code), &mut writer2)?;
                            //     println!("{:?}", format!("http://img1.money.126.net/data/hs/time/today/{}.json",stock.code));  
                            //     let json2: Map<String, Value> = serde_json::from_str(&String::from_utf8_lossy(&writer2).to_string())?;
                            //     stock.slice = json2.get("data").unwrap().as_array().unwrap()
                            //         .iter().map(|item| item.as_array().unwrap().get(2).unwrap().as_f64().unwrap())
                            //         .collect();
                            // }
                        }
                        let mut last_refresh = last_refresh_clone.lock().unwrap();
                        *last_refresh = Local::now();
                        *locked_err = String::new();
                    }
                    else {
                        *locked_err = String::from("Server Returns Errors");
                    }
                }
            });
        }
    }

    // get the stock code
    pub fn get_codes(&self) -> String {
        let codes:Vec<String> = self.stocks.lock().unwrap()
            .iter()
            .map(|stock| stock.code.clone())
            .collect();
        codes.join(",")
    }
}

