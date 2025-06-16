use chrono::{Datelike, NaiveDate};
use clap::Args;

// Note: the unwrap in date arithmetic functions should be safe as we are working with valid dates
// and the operations are designed to stay within valid date ranges.

#[derive(Args, Clone, Debug)]
#[group(required = false, multiple = false)]
pub struct SingleDate {
    #[arg(long, default_value_t=today())]
    date: NaiveDate,
    #[arg(long)]
    today: bool,
    #[arg(long)]
    yesterday: bool,
    #[arg(long)]
    tomorrow: bool,
}

impl SingleDate {
    /// Get the date value directly
    pub fn get_date(&self) -> NaiveDate {
        if self.today {
            today()
        } else if self.yesterday {
            yesterday()
        } else if self.tomorrow {
            tomorrow()
        } else {
            self.date
        }
    }
}

#[derive(Args, Clone, Debug)]
#[group()]
pub struct StartEndDates {
    #[arg(long, default_value_t=first_of_current_month())]
    start_date: NaiveDate,
    #[arg(long, default_value_t=last_of_current_month())]
    end_date: NaiveDate,
}

#[derive(Args, Clone, Debug)]
#[group(required = false, multiple = false)]
pub struct DateRange {
    #[command(flatten)]
    start_end: StartEndDates,
    #[arg(long, conflicts_with_all = ["last_month", "this_year", "last_year", "start_date", "end_date"])]
    this_month: bool,
    #[arg(long, conflicts_with_all = ["this_month", "this_year", "last_year", "start_date", "end_date"])]
    last_month: bool,
    #[arg(long, conflicts_with_all = ["this_month", "last_month", "last_year", "start_date", "end_date"])]
    this_year: bool,
    #[arg(long, conflicts_with_all = ["this_month", "last_month", "this_year", "start_date", "end_date"])]
    last_year: bool,
}

impl DateRange {
    pub fn range(&self) -> (NaiveDate, NaiveDate) {
        if self.this_month {
            return (first_of_current_month(), last_of_current_month());
        } else if self.last_month {
            return (first_of_last_month(), last_of_last_month());
        } else if self.this_year {
            return (first_of_current_year(), last_of_current_year());
        } else if self.last_year {
            return (first_of_last_year(), last_of_last_year());
        }
        (self.start_end.start_date, self.start_end.end_date)
    }
}

fn today() -> NaiveDate {
    chrono::Utc::now().naive_utc().date()
}
fn tomorrow() -> NaiveDate {
    today() + chrono::Duration::days(1)
}
fn yesterday() -> NaiveDate {
    today() - chrono::Duration::days(1)
}

fn first_of_current_month() -> NaiveDate {
    let today = today();
    today.with_day(1).unwrap()
}

fn first_of_last_month() -> NaiveDate {
    let today = today();
    today
        .with_day(1)
        .unwrap()
        .pred_opt()
        .unwrap()
        .with_day(1)
        .unwrap() // Get the last day of the previous month
}
fn first_of_current_year() -> NaiveDate {
    let today = today();
    today.with_day(1).unwrap().with_month(1).unwrap()
}
fn last_of_current_year() -> NaiveDate {
    let today = today();
    today.with_month(12).unwrap().with_day(31).unwrap()
}

fn last_of_current_month() -> NaiveDate {
    let today = today();
    let month = today.month();
    if month == 12 {
        return today.with_day(31).unwrap(); // December has 31 days
    }
    today
        .with_day(1)
        .unwrap()
        .with_month(today.month() + 1)
        .unwrap()
        .pred_opt()
        .unwrap() // Get the last day of the current month
}

fn first_of_last_year() -> NaiveDate {
    let today = today();
    today
        .with_day(1)
        .unwrap()
        .with_month(1)
        .unwrap()
        .with_year(today.year() - 1)
        .unwrap()
}

fn last_of_last_year() -> NaiveDate {
    let today = today();
    today
        .with_month(12)
        .unwrap()
        .with_day(31)
        .unwrap()
        .with_year(today.year() - 1)
        .unwrap()
}

fn last_of_last_month() -> NaiveDate {
    let today = today();
    today.with_day(1).unwrap().pred_opt().unwrap() // Get the last day of the previous month
}
