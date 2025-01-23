use clap::Parser;
use rocket::http::Status;
use rocket::serde::json::{Json, Value};
use rocket::serde::{Deserialize, Deserializer, Serialize};
use rocket::tokio::sync::RwLock;
use rocket::{get, routes};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiningHallInfo {
    pub opening_hours: String,
    pub closing_hours: String,
    pub location_title: String,
    pub breakfast_open_time: Option<String>,
    pub breakfast_close_time: Option<String>,
    pub breakfast_menu: String,
    pub lunch_open_time: Option<String>,
    pub lunch_close_time: Option<String>,
    pub lunch_menu: String,
    pub dinner_open_time: Option<String>,
    pub dinner_close_time: Option<String>,
    pub dinner_menu: String,
    pub latenight_menu: Option<String>,
    pub locations: String,
    pub new_location_hour: Option<NewLocationHour>,
    pub livestream_entrance_link: Option<String>,
    pub livestream_entrance_text: Option<String>,
    pub livestream_seating_link: Option<String>,
    pub livestream_seating_text: Option<String>,
    pub cash_period_start: Option<String>,
    pub cash_period_end: Option<String>,
    pub cash_period: Option<String>,
    pub location_id: i64,
    pub short_name: String,
    pub short_description: String,
    pub short_description_v2: String,
    pub location_url: String,
    pub business_level: i64,
    pub accepted_payment: String,
    pub is_new: String,
    pub distance: Option<String>,
    pub notbusy_level: Value,
    pub moderate_level: Value,
    pub address: String,
    pub map_address: String,
    pub contact_information: String,
    pub contact_information_plain: String,
    #[serde(default, deserialize_with = "deserialize_menu")]
    pub menu: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_menu_meal")]
    pub menu_meal: Option<HashMap<String, Vec<String>>>,
    pub featured_image: String,
    pub open_24: Option<i64>,
    pub mon_hour: Option<String>,
    pub tue_hour: Option<String>,
    pub wed_hour: Option<String>,
    pub thu_hour: Option<String>,
    pub fri_hour: Option<String>,
    pub sat_hour: Option<String>,
    pub sun_hour: Option<String>,
    pub reservation_information: Option<String>,
    pub reservation_information_plain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Menu {
    Bool(bool),
    Menu(String),
}
fn deserialize_menu<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        Some(serde_json::Value::String(json_str)) => {
            // If it's a string, try parsing it as a JSON list of strings
            serde_json::from_str(&json_str)
                .map(Some)
                .map_err(Error::custom)
        }
        Some(serde_json::Value::Bool(_)) => Ok(None), // Treat a bool as None
        Some(serde_json::Value::Null) => Ok(None),    // Treat null as None
        None => Ok(None),                             // Field is missing
        _ => Err(Error::custom("Invalid type for menu field")),
    }
}
fn deserialize_menu_meal<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, Vec<String>>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        Some(serde_json::Value::String(json_str)) => {
            // Try parsing the string as a HashMap<String, Vec<String>>
            serde_json::from_str(&json_str)
                .map(Some)
                .map_err(Error::custom)
        }
        Some(serde_json::Value::Bool(_)) => Ok(None), // Treat a bool as None
        Some(serde_json::Value::Null) => Ok(None),    // Treat null as None
        None => Ok(None),                             // Field is missing
        _ => Err(Error::custom("Invalid type for menu_meal field")),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MenuMeal {
    Bool(bool),
    MenuMeal(String),
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewLocationHour {
    pub exception_title: String,
    pub exceptions: Vec<Exception>,
    pub normal_hour: String,
    pub hours: Vec<Hour>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    pub date: String,
    pub day: String,
    pub hour: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hour {
    pub location_name: String,
    pub day: String,
    pub hour: String,
}

type SharedState = Arc<RwLock<HashMap<String, DiningHallInfo>>>;

#[get("/")]
async fn list_dining_halls(state: &rocket::State<SharedState>) -> Json<Vec<String>> {
    let data = state.read().await;
    let hall_names: Vec<String> = data.keys().cloned().collect();
    Json(hall_names.clone())
}

#[get("/info?<name>")]
async fn get_info(
    state: &rocket::State<SharedState>,
    name: &str,
) -> Result<Json<DiningHallInfo>, Status> {
    let data = state.read().await;
    if let Some(info) = data.get(name) {
        Ok(Json(info.clone()))
    } else {
        Err(Status::NotFound)
    }
}

async fn fetch_dining_hall_data(state: SharedState) {
    let client = reqwest::Client::new();
    loop {
        match client
            .get("https://umassdining.com/uapp/get_infov2")
            .send()
            .await
        {
            Ok(response) => match response.json::<Vec<DiningHallInfo>>().await {
                Ok(data) => {
                    let mut map = HashMap::new();
                    for hall in data {
                        map.insert(hall.location_title.clone(), hall);
                    }
                    *state.write().await = map;
                    println!("Successfully updated dining hall data.");
                }
                Err(err) => {
                    eprintln!("Error parsing JSON response: {:?}", err);
                }
            },
            Err(err) => {
                eprintln!("Error fetching data: {}", err);
            }
        }
        // Wait for 1 hour before retrying
        tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IP address to bind to
    #[arg(long, default_value = "127.0.0.1")]
    ip: String,

    /// Port to bind to
    #[arg(long, default_value_t = 9999)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Parse CLI args
    let args = Args::parse();
    // Set up shared state
    let state: SharedState = Arc::new(RwLock::new(HashMap::new()));
    let data_state = Arc::clone(&state);

    // Start the periodic data fetch task
    tokio::spawn(async move {
        fetch_dining_hall_data(data_state).await;
    });

    // Launch the Rocket server
    rocket::custom(
        rocket::Config::figment()
            .merge(("address", args.ip))
            .merge(("port", args.port)),
    )
    .manage(state)
    .mount("/", routes![get_info, list_dining_halls])
    .launch()
    .await
    .unwrap();
}
