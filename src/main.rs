use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

const SCHEDULE_FILE: &str = "schedule.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Schedule {
    id: u64,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

impl Schedule {
    fn new(id: u64, subject: String, start: NaiveDateTime, end: NaiveDateTime) -> Self {
        Self {
            id,
            subject,
            start,
            end,
        }
    }

    fn intersects(&self, other: &Schedule) -> bool {
        self.start < other.end && other.start < self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Calendar {
    schedules: Vec<Schedule>,
}

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 予定の一覧表示
    List,
    /// 予定の追加
    Add {
        /// タイトル
        subject: String,
        /// 開始日時
        start: NaiveDateTime,
        /// 終了日時
        end: NaiveDateTime,
    },
    /// 予定の削除
    Delete { id: u64 },
}

#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("ファイル操作でエラーが発生しました: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 操作でエラーが発生しました: {0}")]
    Json(#[from] serde_json::Error),
    #[error("予定が重複しています")]
    ScheduleConflict,
    #[error("予定が見つかりませんでした: (ID: {0})")]
    ScheduleNotFound(u64),
}

fn main() {
    let options = App::parse();

    let result = match options.command {
        Commands::List => list_command(),
        Commands::Add {
            subject,
            start,
            end,
        } => add_command(subject, start, end),
        Commands::Delete { id } => delete_command(id),
    };

    if let Err(e) = result {
        eprintln!("エラー: {}", e);
        std::process::exit(1);
    }
}

fn list_command() -> Result<(), MyError> {
    let calendar = read_calendar()?;
    show_list(&calendar);
    Ok(())
}

fn add_command(subject: String, start: NaiveDateTime, end: NaiveDateTime) -> Result<(), MyError> {
    let mut calendar = read_calendar()?;
    add_schedule(&mut calendar, subject, start, end)?;
    save_calendar(&calendar)?;
    println!("予定を追加しました！");
    Ok(())
}

fn delete_command(id: u64) -> Result<(), MyError> {
    let mut calendar = read_calendar()?;
    delete_schedule(&mut calendar, id)?;
    save_calendar(&calendar)?;
    println!("予定を削除しました！");
    Ok(())
}

fn read_calendar() -> Result<Calendar, MyError> {
    match File::open(SCHEDULE_FILE) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let calendar = serde_json::from_reader(reader)?;
            Ok(calendar)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // ファイルが存在しない場合は空のカレンダーを作成して保存
            let calendar = Calendar {
                schedules: Vec::new(),
            };
            save_calendar(&calendar)?;
            Ok(calendar)
        }
        Err(e) => Err(MyError::Io(e)),
    }
}

fn save_calendar(calendar: &Calendar) -> Result<(), MyError> {
    let file = File::create(SCHEDULE_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &calendar)?;
    Ok(())
}

fn delete_schedule(calendar: &mut Calendar, id: u64) -> Result<(), MyError> {
    for i in 0..calendar.schedules.len() {
        if calendar.schedules[i].id == id {
            calendar.schedules.remove(i);
            return Ok(());
        }
    }
    Err(MyError::ScheduleNotFound(id))
}

// 予定の一覧を表示する
fn show_list(calendar: &Calendar) {
    println!("ID\tSTART\t\t\tEND\t\t\tSUBJECT");
    for schedule in &calendar.schedules {
        println!(
            "{}\t{}\t{}\t{}",
            schedule.id, schedule.start, schedule.end, schedule.subject
        );
    }
}

// 予定を追加する
fn add_schedule(
    calendar: &mut Calendar,
    subject: String,
    start: NaiveDateTime,
    end: NaiveDateTime,
) -> Result<(), MyError> {
    let id = calendar.schedules.len() as u64;
    let new_schedule = Schedule::new(id, subject, start, end);

    for schedule in &calendar.schedules {
        if schedule.intersects(&new_schedule) {
            return Err(MyError::ScheduleConflict);
        }
    }

    calendar.schedules.push(new_schedule);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn naive_date_time(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap()
    }

    #[rstest]
    #[case(18, 15, 19, 15, true)]
    #[case(19, 45, 20, 45, true)]
    #[case(18, 30, 20, 15, true)]
    #[case(20, 15, 20, 45, false)]
    #[case(18, 15, 18, 45, false)]
    #[case(19, 15, 19, 45, true)]
    #[case(19, 0, 20, 0, true)] // 境界値テスト
    fn test_schedule_intersects(
        #[case] h0: u32,
        #[case] m0: u32,
        #[case] h1: u32,
        #[case] m1: u32,
        #[case] expected: bool,
    ) {
        let schedule = Schedule {
            id: 0,
            subject: "既存予定".to_string(),
            start: naive_date_time(2024, 1, 1, h0, m0, 0),
            end: naive_date_time(2024, 1, 1, h1, m1, 0),
        };

        let new_schedule = Schedule {
            id: 1,
            subject: "新規予定".to_string(),
            start: naive_date_time(2024, 1, 1, 19, 0, 0),
            end: naive_date_time(2024, 1, 1, 20, 0, 0),
        };

        assert_eq!(schedule.intersects(&new_schedule), expected);
    }

    #[test]
    fn test_delete_schedule() {
        let mut calendar = Calendar {
            schedules: vec![
                Schedule::new(
                    0,
                    "既存予定".to_string(),
                    naive_date_time(2024, 1, 1, 18, 15, 0),
                    naive_date_time(2024, 1, 1, 19, 15, 0),
                ),
                Schedule::new(
                    1,
                    "既存予定".to_string(),
                    naive_date_time(2024, 1, 1, 19, 45, 0),
                    naive_date_time(2024, 1, 1, 20, 45, 0),
                ),
                Schedule::new(
                    2,
                    "既存予定".to_string(),
                    naive_date_time(2024, 1, 1, 20, 15, 0),
                    naive_date_time(2024, 1, 1, 21, 15, 0),
                ),
            ],
        };

        // id = 0 の予定を削除
        assert!(delete_schedule(&mut calendar, 0).is_ok());

        let expected = Calendar {
            schedules: vec![
                Schedule::new(
                    1,
                    "既存予定".to_string(),
                    naive_date_time(2024, 1, 1, 19, 45, 0),
                    naive_date_time(2024, 1, 1, 20, 45, 0),
                ),
                Schedule::new(
                    2,
                    "既存予定".to_string(),
                    naive_date_time(2024, 1, 1, 20, 15, 0),
                    naive_date_time(2024, 1, 1, 21, 15, 0),
                ),
            ],
        };

        assert_eq!(expected, calendar);
        // id = 1 の予定を削除
        assert!(delete_schedule(&mut calendar, 1).is_ok());

        let expected = Calendar {
            schedules: vec![Schedule::new(
                2,
                "既存予定".to_string(),
                naive_date_time(2024, 1, 1, 20, 15, 0),
                naive_date_time(2024, 1, 1, 21, 15, 0),
            )],
        };

        assert_eq!(expected, calendar);

        // id = 2 の予定を削除
        assert!(delete_schedule(&mut calendar, 2).is_ok());

        let expected = Calendar { schedules: vec![] };
        assert_eq!(expected, calendar);
    }
}
