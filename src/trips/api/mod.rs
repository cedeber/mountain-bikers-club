use serde::Deserialize;

pub mod post_delete;
pub mod post_gpx;
pub mod post_join;
pub mod post_new;
pub mod post_reconsider;
pub mod post_update;

pub fn deserialize_option_ignore_error<'de, T, D>(d: D) -> Result<Option<T>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(T::deserialize(d).ok())
}

#[derive(Debug, Deserialize)]
pub struct NewForm {
    name: String,
    description: Option<String>,
    day: u32,
    month: u32,
    year: u32,
    hour: u32,
    minute: u32,
    timezone_diff: i64,
    meeting_point: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    time_hour: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    time_minute: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    distance: Option<f32>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    elevation: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateForm {
    trip_id: i32,
    name: String,
    description: Option<String>,
    day: u32,
    month: u32,
    year: u32,
    hour: u32,
    minute: u32,
    timezone_diff: i64,
    meeting_point: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    time_hour: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    time_minute: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    distance: Option<f32>,
    #[serde(default, deserialize_with = "deserialize_option_ignore_error")]
    elevation: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct DeleteForm {
    trip_id: i32,
    redirect_trip_username: String,
    redirect_trip_uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinForm {
    trip_id: i32,
    will_join: bool,
    redirect_trip_username: String,
    redirect_trip_uuid: String,
}
