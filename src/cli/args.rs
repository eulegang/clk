use chrono::{prelude::*, DateTime, Local, MappedLocalTime};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp(DateTime<Local>);

impl std::str::FromStr for Timestamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s
            .parse::<NaiveDateTime>()
            .map_err(|err| format!("{s:?} is not a date time: {err}"))?;

        let inner = match Local.from_local_datetime(&inner) {
            MappedLocalTime::Single(inner) => inner,
            _ => {
                return Err(format!("unable to convert date to localtime"));
            }
        };

        Ok(Timestamp(inner))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Duration {
    seconds: u64,
}

impl std::str::FromStr for Duration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split(":").collect::<Vec<_>>();

        if parts.len() > 3 {
            return Err(format!("{s:?} has to many components"));
        }

        let mut seconds = 0;
        for part in parts {
            seconds *= 60;

            let Ok(t) = part.parse::<u64>() else {
                return Err(format!("{s:?} is invalid because {part:?} is not a number"));
            };

            if t >= 60 {
                return Err(format!("{s:?} is invalid because {t} >= 60"));
            }

            seconds += t;
        }

        Ok(Duration { seconds })
    }
}

#[test]
fn test_parse_duration() {
    assert_eq!("3".parse::<Duration>(), Ok(Duration { seconds: 3 }));
    assert_eq!("55".parse::<Duration>(), Ok(Duration { seconds: 55 }));
    assert_eq!(
        "65".parse::<Duration>(),
        Err("\"65\" is invalid because 65 >= 60".to_string())
    );
    assert_eq!(
        "65:30".parse::<Duration>(),
        Err("\"65:30\" is invalid because 65 >= 60".to_string())
    );
    assert_eq!(
        "04:05:06:30".parse::<Duration>(),
        Err("\"04:05:06:30\" has to many components".to_string())
    );
    assert_eq!(
        "05:06:30".parse::<Duration>(),
        Ok(Duration {
            seconds: 5 * 3600 + 6 * 60 + 30
        })
    );
}

#[test]
fn test_parse_timestamp() {
    assert_eq!(
        "2024-9-23T16:30:00".parse::<Timestamp>(),
        Ok(Timestamp(
            Local.with_ymd_and_hms(2024, 09, 23, 16, 30, 0).unwrap()
        ))
    );
}

impl From<Timestamp> for DateTime<Local> {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl From<Duration> for chrono::Duration {
    fn from(value: Duration) -> Self {
        chrono::Duration::new(value.seconds as i64, 0).unwrap()
    }
}
