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

fn main() {
    let options = App::parse();

    match options.command {
        Commands::List => show_list(),
        Commands::Add {
            subject,
            start,
            end,
        } => add_schedule(subject, start, end),
        Commands::Delete { id } => {
            let mut calendar = read_calendar();
            if delete_schedule(&mut calendar, id) {
                save_calendar(&calendar);
            } else {
                println!("予定が見つかりませんでした");
            }
        }
    }
}

fn read_calendar() -> Calendar {
    let file = File::open(SCHEDULE_FILE).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn save_calendar(calendar: &Calendar) {
    let file = File::create(SCHEDULE_FILE).unwrap();
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &calendar).unwrap();
}

fn show_list() {
    let calendar = read_calendar();

    // 予定の表示
    println!("ID\tSTART\tEND\tSUBJECT");
    for schedule in calendar.schedules {
        println!(
            "{}\t{}\t{}\t{}",
            schedule.id, schedule.start, schedule.end, schedule.subject
        );
    }
}

fn add_schedule(subject: String, start: NaiveDateTime, end: NaiveDateTime) {
    let mut calendar = read_calendar();

    // 予定の作成
    let id = calendar.schedules.len() as u64;
    let new_schedule = Schedule::new(id, subject, start, end);

    for schedule in &calendar.schedules {
        if schedule.intersects(&new_schedule) {
            println!("エラー：予定が重複しています");
            return;
        }
    }

    calendar.schedules.push(new_schedule);

    // 予定の保存
    {
        save_calendar(&calendar);
    }

    println!("予定を追加しました！");
}

fn delete_schedule(calendar: &mut Calendar, id: u64) -> bool {
    for i in 0..calendar.schedules.len() {
        if calendar.schedules[i].id == id {
            calendar.schedules.remove(i);
            return true;
        }
    }
    false
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
        assert!(delete_schedule(&mut calendar, 0));

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
        assert!(delete_schedule(&mut calendar, 1));

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
        assert!(delete_schedule(&mut calendar, 2));

        let expected = Calendar { schedules: vec![] };
        assert_eq!(expected, calendar);
    }
}
