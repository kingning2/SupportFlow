use std::time::{SystemTime, UNIX_EPOCH};

const UTC8_OFFSET_SECS: u64 = 8 * 3600;

fn current_unix_secs_utc8() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now.saturating_add(UTC8_OFFSET_SECS)
}

/// 获取当前 Unix 秒级时间戳。
pub fn unix_timestamp_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取当前 Unix 秒级时间戳字符串，适合写入索引文件。
pub fn unix_timestamp_string() -> String {
    unix_timestamp_seconds().to_string()
}

/// 判断闰年
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// 获取当前日期字符串 YYYY-MM-DD
pub fn current_date_string() -> String {
    let now_secs = current_unix_secs_utc8();

    // Unix 时间戳秒数
    let days_since_epoch = now_secs / 86_400;

    // 1970-01-01 为 epoch，转换成年月日
    let mut year = 1970;
    let mut days = days_since_epoch as i32;

    while days >= if is_leap_year(year) { 366 } else { 365 } {
        days -= if is_leap_year(year) { 366 } else { 365 };
        year += 1;
    }

    let month_days = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0;
    while days >= month_days[month] {
        days -= month_days[month];
        month += 1;
    }

    let day = days + 1;

    format!("{:04}-{:02}-{:02}", year, month + 1, day)
}

/// 获取当前时间字符串 HH:mm:ss
pub fn current_hms_string() -> String {
    let total_secs = current_unix_secs_utc8() % 86_400; // 一天 86400 秒
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;

    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// 获取当前日期时间字符串 YYYY-MM-DD HH:mm:ss
pub fn current_datetime_string() -> String {
    format!("{} {}", current_date_string(), current_hms_string())
}
