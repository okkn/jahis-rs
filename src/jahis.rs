// https://www.jahis.jp/standard/detail/id=665
// https://www.jahis.jp/files/user/04_JAHIS%20standard/19-102_JAHIS%E9%9B%BB%E5%AD%90%E7%89%88%E3%81%8A%E8%96%AC%E6%89%8B%E5%B8%B3%E3%83%87%E3%83%BC%E3%82%BF%E3%83%95%E3%82%A9%E3%83%BC%E3%83%9E%E3%83%83%E3%83%88%E4%BB%95%E6%A7%98%E6%9B%B8Ver.2.3.pdf

#![allow(dead_code)]

use std::num;
use std::fmt;
use std::str::FromStr;
use std::convert::From;
use std::convert::TryFrom;
use lazy_static::lazy_static;
use chrono;
use chrono::Datelike;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An error which can be return when parsing a date string.
pub enum Error {
    InvalidArgument(String),
    InvalidRecordLine(String),
    GotUnexpectedRecordLine(String),
    MissingRequiredRecord(String),
    Unreachable(String),
    ParseIntError(num::ParseIntError),
    ParseFloatError(num::ParseFloatError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Appended Table 1: Japanese era calendar scheme (Gengo)
pub enum GengoYear {
    Reiwa(i32), // 令和
    Heisei(i32), // 平成
    Showa(i32), // 昭和
    Taisho(i32), // 大正
    Meiji(i32), // 明示
}

impl GengoYear {
    pub fn to_code(&self) -> String {
        match *self {
            Self::Reiwa(y) => format!("R{:>02}", y),
            Self::Heisei(y) => format!("H{:>02}", y),
            Self::Showa(y) => format!("S{:>02}", y),
            Self::Taisho(y) => format!("T{:>02}", y),
            Self::Meiji(y) => format!("M{:>02}", y),
        }
    }
}

impl fmt::Display for GengoYear {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Reiwa(y) => write!(f, "令和{}年", if y == 1 {"元".to_string()} else {y.to_string()}),
            Self::Heisei(y) => write!(f, "平成{}年", if y == 1 {"元".to_string()} else {y.to_string()}),
            Self::Showa(y) => write!(f, "昭和{}年", if y == 1 {"元".to_string()} else {y.to_string()}),
            Self::Taisho(y) => write!(f, "大正{}年", if y == 1 {"元".to_string()} else {y.to_string()}),
            Self::Meiji(y) => write!(f, "明治{}年", if y == 1 {"元".to_string()} else {y.to_string()}),
        }
    }
}

impl FromStr for GengoYear {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^([RrHhSsTtMm㋿㍻㍼㍽㍾]|令和|平成|昭和|大正|明治)(\d+|元)年?$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            let year: i32 = if &cap[2] == "元" {1} else {(&cap[2]).parse().map_err(Error::ParseIntError)?};
            match &cap[1] {
                "令和" | "㋿" | "R" | "r" => return Ok(Self::Reiwa(year)),
                "平成" | "㍻" | "H" | "h" => return Ok(Self::Heisei(year)),
                "昭和" | "㍼" | "S" | "s" => return Ok(Self::Showa(year)),
                "大正" | "㍽" | "T" | "t" => return Ok(Self::Taisho(year)),
                "明治" | "㍾" | "M" | "m" => return Ok(Self::Meiji(year)),
                _ => return Err(Error::Unreachable(
                    format!("Unreachable code in GengoYear, got \"{}\"", s)
                )),
            }
        }
        Err(Error::InvalidArgument(
            format!("Cannot convert str to GengoYear, got \"{}\"", s)
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A struct that holds date in seireki or in wareki
pub enum Date {
    Seireki{year: i32, month: u32, day: u32},
    Wareki{gengo_year: GengoYear, month: u32, day: u32},
}

impl Date {
    pub fn to_naivedate(&self) -> chrono::NaiveDate {
        match *self {
            Self::Seireki{year: y, month: m, day: d} => chrono::NaiveDate::from_ymd(y, m, d),
            Self::Wareki{gengo_year: gy, month: m, day: d} => {
                match gy {
                    GengoYear::Reiwa(y) => chrono::NaiveDate::from_ymd(y + 2018, m, d),
                    GengoYear::Heisei(y) => chrono::NaiveDate::from_ymd(y + 1988, m, d),
                    GengoYear::Showa(y) => chrono::NaiveDate::from_ymd(y + 1925, m, d),
                    GengoYear::Taisho(y) => chrono::NaiveDate::from_ymd(y + 1911, m, d),
                    GengoYear::Meiji(y) => chrono::NaiveDate::from_ymd(y + 1867, m, d),
                }
            }
        }
    }

    pub fn to_code(&self) -> String {
        match *self {
            Self::Seireki{year: y, month: m, day: d} => format!("{:>04}{:>02}{:>02}", y, m, d),
            Self::Wareki{gengo_year: gy, month: m, day: d} => format!("{}{:>02}{:>02}", gy.to_code(), m, d),
        }
    }

    pub fn to_seireki8(&self) -> String {
        match *self {
            Self::Seireki{year: y, month: m, day: d} => format!("{:>04}{:>02}{:>02}", y, m, d),
            Self::Wareki{gengo_year: gy, month: m, day: d} => {
                match gy {
                    GengoYear::Reiwa(y) => format!("{:>04}{:>02}{:>02}", y + 2018, m, d),
                    GengoYear::Heisei(y) => format!("{:>04}{:>02}{:>02}", y + 1988, m, d),
                    GengoYear::Showa(y) => format!("{:>04}{:>02}{:>02}", y + 1925, m, d),
                    GengoYear::Taisho(y) => format!("{:>04}{:>02}{:>02}", y + 1911, m, d),
                    GengoYear::Meiji(y) => format!("{:>04}{:>02}{:>02}", y + 1867, m, d),
                }
            }
        }
    }

    pub fn try_to_wareki7(&self) -> Result<String, Error> {
        match *self {
            Self::Seireki{year: y, month: m, day: d} => {
                if y > 2019 || y == 2019 && m >= 5 {
                    return Ok(format!("R{:>02}{:>02}{:>02}", y - 2018, m, d));
                } else if y > 1989 || y == 1989 && m > 1 || y == 1989 && m == 1 && d >= 8 {
                    return Ok(format!("H{:>02}{:>02}{:>02}", y - 1988, m, d));
                } else if y > 1926 || y == 1926 && m == 12 && d >= 25 {
                    return Ok(format!("S{:>02}{:>02}{:>02}", y - 1925, m, d));
                } else if y > 1912 || y == 1912 && m > 7 || y == 1912 && m == 7 && d >= 30 {
                    return Ok(format!("T{:>02}{:>02}{:>02}", y - 1911, m, d));
                } else if y > 1872 {
                    return Ok(format!("M{:>02}{:>02}{:>02}", y - 1867, m, d));
                } else {
                    return Err(Error::InvalidArgument(
                        format!("Cannot convert seireki8 to wareki7, got \"{:?}\"", *self)
                    ));
                }
            },
            Self::Wareki{gengo_year: gy, month: m, day: d} => {
                match gy {
                    GengoYear::Reiwa(y) => Ok(format!("R{:>02}{:>02}{:>02}", y, m, d)),
                    GengoYear::Heisei(y) => Ok(format!("H{:>02}{:>02}{:>02}", y, m, d)),
                    GengoYear::Showa(y) => Ok(format!("S{:>02}{:>02}{:>02}", y, m, d)),
                    GengoYear::Taisho(y) => Ok(format!("T{:>02}{:>02}{:>02}", y, m, d)),
                    GengoYear::Meiji(y) => Ok(format!("M{:>02}{:>02}{:>02}", y, m, d)),
                }
            },
        }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Seireki{year: y, month: m, day: d} => write!(f, "{}年{}月{}日", y, m, d),
            Self::Wareki{gengo_year: gy, month: m, day: d} => write!(f, "{}{}月{}日", gy, m, d),
        }
    }
}

impl FromStr for Date {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE_SEIREKI8: Regex = Regex::new(r"^(\d{4})(\d{2})(\d{2})?$").unwrap();
            static ref RE_WAREKI7: Regex = Regex::new(r"^([RHSTM]\d{2})(\d{2})(\d{2})$").unwrap();
        }
        if RE_SEIREKI8.is_match(s) {
            for cap in RE_SEIREKI8.captures_iter(s) {
                let y: i32 = (&cap[1]).parse().unwrap();
                let m: u32 = (&cap[2]).parse().unwrap();
                let d: u32 = (&cap[3]).parse().unwrap();
                return Ok(Date::Seireki{year: y, month: m, day: d})
            }
        } else if RE_WAREKI7.is_match(s) {
            for cap in RE_WAREKI7.captures_iter(s) {
                let gy: &str = &cap[1];
                let m: u32 = (&cap[2]).parse().unwrap();
                let d: u32 = (&cap[3]).parse().unwrap();
                return Ok(Date::Wareki{gengo_year: gy.parse::<GengoYear>().unwrap(), month: m, day: d})
            }
        }
        Err(Error::InvalidArgument(
            format!("Cannot convert str to Date, got \"{}\"", s)
        ))
    }
}

impl From<chrono::NaiveDate> for Date {
    fn from(d: chrono::NaiveDate) -> Self {
        Self::Seireki{year: d.year(), month: d.month(), day: d.day()}
    }
}
impl From<Date> for chrono::NaiveDate {
    fn from(d: Date) -> Self {
        match d {
            Date::Seireki{year: y, month: m, day: d} => chrono::NaiveDate::from_ymd(y, m, d),
            Date::Wareki{gengo_year: gy, month: m, day: d} => {
                match gy {
                    GengoYear::Reiwa(y) => chrono::NaiveDate::from_ymd(y + 2018, m, d),
                    GengoYear::Heisei(y) => chrono::NaiveDate::from_ymd(y + 1988, m, d),
                    GengoYear::Showa(y) => chrono::NaiveDate::from_ymd(y + 1925, m, d),
                    GengoYear::Taisho(y) => chrono::NaiveDate::from_ymd(y + 1911, m, d),
                    GengoYear::Meiji(y) => chrono::NaiveDate::from_ymd(y + 1867, m, d),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Appended Table 2: Prefecture Code, which confirms to JIS X 0401:1973 and ISO 3166-2:JP
pub enum Prefecture {
    Hokkaido = 1,
    Aomori = 2,
    Iwate = 3,
    Miyagi = 4,
    Akita = 5,
    Yamagata = 6,
    Fukushima = 7,
    Ibaraki = 8,
    Tochigi = 9,
    Gumma = 10,
    Saitama = 11,
    Chiba = 12,
    Tokyo = 13,
    Kanagawa = 14,
    Niigata = 15,
    Toyama = 16,
    Ishikawa = 17,
    Fukui = 18,
    Yamanashi = 19,
    Nagano = 20,
    Gifu = 21,
    Shizuoka = 22,
    Aichi = 23,
    Mie = 24,
    Shiga = 25,
    Kyoto = 26,
    Osaka = 27,
    Hyogo = 28,
    Nara = 29,
    Wakayama = 30,
    Tottori = 31,
    Shimane = 32,
    Okayama = 33,
    Hiroshima = 34,
    Yamaguchi = 35,
    Tokushima = 36,
    Kagawa = 37,
    Ehime = 38,
    Kochi = 39,
    Fukuoka = 40,
    Saga = 41,
    Nagasaki = 42,
    Kumamoto = 43,
    Oita = 44,
    Miyazaki = 45,
    Kagoshima = 46,
    Okinawa = 47,
}

impl Prefecture {
    pub fn to_code(&self) -> String {
        format!("{:>02}", *self as u32)
    }
}

impl fmt::Display for Prefecture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Hokkaido => write!(f, "北海道"),
            Self::Aomori => write!(f, "青森県"),
            Self::Iwate => write!(f, "岩手県"),
            Self::Miyagi => write!(f, "宮城県"),
            Self::Akita => write!(f, "秋田県"),
            Self::Yamagata => write!(f, "山形県"),
            Self::Fukushima => write!(f, "福島県"),
            Self::Ibaraki => write!(f, "茨城県"),
            Self::Tochigi => write!(f, "栃木県"),
            Self::Gumma => write!(f, "群馬県"),
            Self::Saitama => write!(f, "埼玉県"),
            Self::Chiba => write!(f, "千葉県"),
            Self::Tokyo => write!(f, "東京都"),
            Self::Kanagawa => write!(f, "神奈川県"),
            Self::Niigata => write!(f, "新潟県"),
            Self::Toyama => write!(f, "富山県"),
            Self::Ishikawa => write!(f, "石川県"),
            Self::Fukui => write!(f, "福井県"),
            Self::Yamanashi => write!(f, "山梨県"),
            Self::Nagano => write!(f, "長野県"),
            Self::Gifu => write!(f, "岐阜県"),
            Self::Shizuoka => write!(f, "静岡県"),
            Self::Aichi => write!(f, "愛知県"),
            Self::Mie => write!(f, "三重県"),
            Self::Shiga => write!(f, "滋賀県"),
            Self::Kyoto => write!(f, "京都府"),
            Self::Osaka => write!(f, "大阪府"),
            Self::Hyogo => write!(f, "兵庫県"),
            Self::Nara => write!(f, "奈良県"),
            Self::Wakayama => write!(f, "和歌山県"),
            Self::Tottori => write!(f, "鳥取県"),
            Self::Shimane => write!(f, "島根県"),
            Self::Okayama => write!(f, "岡山県"),
            Self::Hiroshima => write!(f, "広島県"),
            Self::Yamaguchi => write!(f, "山口県"),
            Self::Tokushima => write!(f, "徳島県"),
            Self::Kagawa => write!(f, "香川県"),
            Self::Ehime => write!(f, "愛媛県"),
            Self::Kochi => write!(f, "高知県"),
            Self::Fukuoka => write!(f, "福岡県"),
            Self::Saga => write!(f, "佐賀県"),
            Self::Nagasaki => write!(f, "長崎県"),
            Self::Kumamoto => write!(f, "熊本県"),
            Self::Oita => write!(f, "大分県"),
            Self::Miyazaki => write!(f, "宮崎県"),
            Self::Kagoshima => write!(f, "鹿児島県"),
            Self::Okinawa  => write!(f, "沖縄県"),
        }
    }
}

impl FromStr for Prefecture {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "01" | "JP-01" | "北海道" | "Hokkaido" => Ok(Self::Hokkaido),
            "02" | "JP-02" | "青森" | "青森県" | "Aomori" => Ok(Self::Aomori),
            "03" | "JP-03" | "岩手" | "岩手県" | "Iwate" => Ok(Self::Iwate),
            "04" | "JP-04" | "宮城" | "宮城県" | "Miyagi" => Ok(Self::Miyagi),
            "05" | "JP-05" | "秋田" | "秋田県" | "Akita" => Ok(Self::Akita),
            "06" | "JP-06" | "山形" | "山形県" | "Yamagata" => Ok(Self::Yamagata),
            "07" | "JP-07" | "福島" | "福島県" | "Fukushima" => Ok(Self::Fukushima),
            "08" | "JP-08" | "茨城" | "茨城県" | "Ibaraki" => Ok(Self::Ibaraki),
            "09" | "JP-09" | "栃木" | "栃木県" | "Tochigi" => Ok(Self::Tochigi),
            "10" | "JP-10" | "群馬" | "群馬県" | "Gumma" => Ok(Self::Gumma),
            "11" | "JP-11" | "埼玉" | "埼玉県" | "Saitama" => Ok(Self::Saitama),
            "12" | "JP-12" | "千葉" | "千葉県" | "Chiba" => Ok(Self::Chiba),
            "13" | "JP-13" | "東京" | "東京都" | "Tokyo" => Ok(Self::Tokyo),
            "14" | "JP-14" | "神奈川" | "神奈川県" | "Kanagawa" => Ok(Self::Kanagawa),
            "15" | "JP-15" | "新潟" | "新潟県" | "Niigata" => Ok(Self::Niigata),
            "16" | "JP-16" | "富山" | "富山県" | "Toyama" => Ok(Self::Toyama),
            "17" | "JP-17" | "石川" | "石川県" | "Ishikawa" => Ok(Self::Ishikawa),
            "18" | "JP-18" | "福井" | "福井県" | "Fukui" => Ok(Self::Fukui),
            "19" | "JP-19" | "山梨" | "山梨県" | "Yamanashi" => Ok(Self::Yamanashi),
            "20" | "JP-20" | "長野" | "長野県" | "Nagano" => Ok(Self::Nagano),
            "21" | "JP-21" | "岐阜" | "岐阜県" | "Gifu" => Ok(Self::Gifu),
            "22" | "JP-22" | "静岡" | "静岡県" | "Shizuoka" => Ok(Self::Shizuoka),
            "23" | "JP-23" | "愛知" | "愛知県" | "Aichi" => Ok(Self::Aichi),
            "24" | "JP-24" | "三重" | "三重県" | "Mie" => Ok(Self::Mie),
            "25" | "JP-25" | "滋賀" | "滋賀県" | "Shiga" => Ok(Self::Shiga),
            "26" | "JP-26" | "京都" | "京都府" | "Kyoto" => Ok(Self::Kyoto),
            "27" | "JP-27" | "大坂" | "大阪府" | "Osaka" => Ok(Self::Osaka),
            "28" | "JP-28" | "兵庫" | "兵庫県" | "Hyogo" => Ok(Self::Hyogo),
            "29" | "JP-29" | "奈良" | "奈良県" | "Nara" => Ok(Self::Nara),
            "30" | "JP-30" | "和歌山" | "和歌山県" | "Wakayama" => Ok(Self::Wakayama),
            "31" | "JP-31" | "鳥取" | "鳥取県" | "Tottori" => Ok(Self::Tottori),
            "32" | "JP-32" | "島根" | "島根県" | "Shimane" => Ok(Self::Shimane),
            "33" | "JP-33" | "岡山" | "岡山県" | "Okayama" => Ok(Self::Okayama),
            "34" | "JP-34" | "広島" | "広島県" | "Hiroshima" => Ok(Self::Hiroshima),
            "35" | "JP-35" | "山口" | "山口県" | "Yamaguchi" => Ok(Self::Yamaguchi),
            "36" | "JP-36" | "徳島" | "徳島県" | "Tokushima" => Ok(Self::Tokushima),
            "37" | "JP-37" | "香川" | "香川県" | "Kagawa" => Ok(Self::Kagawa),
            "38" | "JP-38" | "愛媛" | "愛媛県" | "Ehime" => Ok(Self::Ehime),
            "39" | "JP-39" | "高知" | "高知県" | "Kochi" => Ok(Self::Kochi),
            "40" | "JP-40" | "福岡" | "福岡県" | "Fukuoka" => Ok(Self::Fukuoka),
            "41" | "JP-41" | "佐賀" | "佐賀県" | "Saga" => Ok(Self::Saga),
            "42" | "JP-42" | "長崎" | "長崎県" | "Nagasaki" => Ok(Self::Nagasaki),
            "43" | "JP-43" | "熊本" | "熊本県" | "Kumamoto" => Ok(Self::Kumamoto),
            "44" | "JP-44" | "大分" | "大分県" | "Oita" => Ok(Self::Oita),
            "45" | "JP-45" | "宮崎" | "宮崎県" | "Miyazaki" => Ok(Self::Miyazaki),
            "46" | "JP-46" | "鹿児島" | "鹿児島県" | "Kagoshima" => Ok(Self::Kagoshima),
            "47" | "JP-47" | "沖縄" | "沖縄県" | "Okinawa" => Ok(Self::Okinawa),
            _ => Err(Error::InvalidArgument(
                format!("Cannot covert str to Prefecture, got\"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for Prefecture {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::Hokkaido),
            2 => Ok(Self::Aomori),
            3 => Ok(Self::Iwate),
            4 => Ok(Self::Miyagi),
            5 => Ok(Self::Akita),
            6 => Ok(Self::Yamagata),
            7 => Ok(Self::Fukushima),
            8 => Ok(Self::Ibaraki),
            9 => Ok(Self::Tochigi),
            10 => Ok(Self::Gumma),
            11 => Ok(Self::Saitama),
            12 => Ok(Self::Chiba),
            13 => Ok(Self::Tokyo),
            14 => Ok(Self::Kanagawa),
            15 => Ok(Self::Niigata),
            16 => Ok(Self::Toyama),
            17 => Ok(Self::Ishikawa),
            18 => Ok(Self::Fukui),
            19 => Ok(Self::Yamanashi),
            20 => Ok(Self::Nagano),
            21 => Ok(Self::Gifu),
            22 => Ok(Self::Shizuoka),
            23 => Ok(Self::Aichi),
            24 => Ok(Self::Mie),
            25 => Ok(Self::Shiga),
            26 => Ok(Self::Kyoto),
            27 => Ok(Self::Osaka),
            28 => Ok(Self::Hyogo),
            29 => Ok(Self::Nara),
            30 => Ok(Self::Wakayama),
            31 => Ok(Self::Tottori),
            32 => Ok(Self::Shimane),
            33 => Ok(Self::Okayama),
            34 => Ok(Self::Hiroshima),
            35 => Ok(Self::Yamaguchi),
            36 => Ok(Self::Tokushima),
            37 => Ok(Self::Kagawa),
            38 => Ok(Self::Ehime),
            39 => Ok(Self::Kochi),
            40 => Ok(Self::Fukuoka),
            41 => Ok(Self::Saga),
            42 => Ok(Self::Nagasaki),
            43 => Ok(Self::Kumamoto),
            44 => Ok(Self::Oita),
            45 => Ok(Self::Miyazaki),
            46 => Ok(Self::Kagoshima),
            47 => Ok(Self::Okinawa),
            _ => Err(Error::InvalidArgument(
                format!("Cannot covert u32 to Prefecture, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Appended Table 3: Type of Medical Fee Table
pub enum FeeTable {
    Medicine = 1, // 医科
    Dentistry = 3, // 歯科
    Pharmacy = 4, // 調剤
}

impl FeeTable {
    pub fn to_code(&self) -> String {
        format!("{}", *self as u32)
    }
}

impl fmt::Display for FeeTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Medicine => write!(f, "医科"),
            Self::Dentistry => write!(f, "歯科"),
            Self::Pharmacy => write!(f, "調剤"),
        }
    }
}

impl FromStr for FeeTable {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "医科" => Ok(Self::Medicine),
            "3" | "歯科" => Ok(Self::Dentistry),
            "4" | "調剤" => Ok(Self::Pharmacy),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to FeeTable, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for FeeTable {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::Medicine),
            3 => Ok(Self::Dentistry),
            4 => Ok(Self::Pharmacy),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to FeeTable, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Appended Table 4: Type of dosage form
pub enum DosageForm {
    OralAdministration = 1, // 内服
    Drop = 2, // 内滴
    Potion =3, // 頓服
    Injection = 4, // 注射
    ExternalUse = 5, // 外用
    Infusodecoction = 6, // 浸煎
    Decoction = 7, // 湯
    Material = 9, // 材料
    Other = 10, // その他
}

impl DosageForm {
    pub fn to_code(&self) -> String {
        format!("{}", *self as u32)
    }
}

impl fmt::Display for DosageForm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::OralAdministration => write!(f, "内服"),
            Self::Drop => write!(f, "内滴"),
            Self::Potion => write!(f, "頓服"),
            Self::Injection => write!(f, "注射"),
            Self::ExternalUse => write!(f, "外用"),
            Self::Infusodecoction => write!(f, "浸煎"),
            Self::Decoction => write!(f, "湯"),
            Self::Material => write!(f, "材料"),
            Self::Other => write!(f, "その他"),
        }
    }
}

impl FromStr for DosageForm {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "内服" => Ok(Self::OralAdministration),
            "2" | "内滴" => Ok(Self::Drop),
            "3" | "頓服" => Ok(Self::Potion),
            "4" | "注射" => Ok(Self::Injection),
            "5" | "外用" => Ok(Self::ExternalUse),
            "6" | "浸煎" => Ok(Self::Infusodecoction),
            "7" | "湯" => Ok(Self::Decoction),
            "9" | "材料" => Ok(Self::Material),
            "10" | "その他" => Ok(Self::Other),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to DosageForm, got\"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for DosageForm {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::OralAdministration),
            2 => Ok(Self::Drop),
            3 => Ok(Self::Potion),
            4 => Ok(Self::Injection),
            5 => Ok(Self::ExternalUse),
            6 => Ok(Self::Infusodecoction),
            7 => Ok(Self::Decoction),
            9 => Ok(Self::Material),
            10 => Ok(Self::Other),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to DosageForm, got \"{}\"", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecordCreator {
    MedicalExpert = 1, // 医療関係者
    Patient = 2, // 患者等
    Other = 8, // その他
    Unknown = 9, // 不明
}

impl RecordCreator {
    pub fn to_code(&self) -> String {
        format!("{}", *self as u32)
    }
}

impl fmt::Display for RecordCreator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::MedicalExpert => write!(f, "医療関係者"),
            Self::Patient => write!(f, "患者等"),
            Self::Other => write!(f, "その他"),
            Self::Unknown => write!(f, "不明"),
        }
    }
}

impl FromStr for RecordCreator {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "医療関係者" => Ok(Self::MedicalExpert),
            "2" | "患者等" | "患者など" | "患者" => Ok(Self::Patient),
            "8" | "その他" => Ok(Self::Other),
            "9" | "不明" => Ok(Self::Unknown),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to RecordCreator, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for RecordCreator {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::MedicalExpert),
            2 => Ok(Self::Patient),
            8 => Ok(Self::Other),
            9 => Ok(Self::Unknown),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to RecordCreator, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputCategory {
    ToPatient = 1, // 医療機関・薬局から患者等に情報を提供する場合
    FromPatinet = 2, // 患者等から医療機関・薬局に情報を提供する場合
}

impl OutputCategory {
    pub fn to_code(&self) -> String {
        format!("{}", *self as u32)
    }
}

impl fmt::Display for OutputCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ToPatient => write!(f, "医療機関・薬局から患者等に情報を提供する場合"),
            Self::FromPatinet => write!(f, "患者等から医療機関・薬局に情報を提供する場合"),
        }
    }
}

impl FromStr for OutputCategory {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::ToPatient),
            "2" => Ok(Self::FromPatinet),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to OutputCategory, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for OutputCategory {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::ToPatient),
            2 => Ok(Self::FromPatinet),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to OutputCategory, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Male = 1, // 男性
    Female = 2, // 女性
}

impl Gender {
    pub fn to_code(&self) -> String {
        format!("{}", *self as u32)
    }
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Male => write!(f, "男性"),
            Self::Female => write!(f, "女性"),
        }
    }
}

impl FromStr for Gender {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "男" | "男性" | "male" => Ok(Self::Male),
            "2" | "女" | "女性" | "female" => Ok(Self::Female),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to Gender, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for Gender {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::Male),
            2 => Ok(Self::Female),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to Gender, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialPatientNoteCategory {
    Allergy = 1, // アレルギー歴
    AdverseEvent = 2, // 副作用歴
    PastHistory = 3, // 既往歴
    Other = 9, // その他
}

impl SpecialPatientNoteCategory {
    pub fn to_code(&self) -> String {
        format!("{}", *self as u32)
    }
}

impl fmt::Display for SpecialPatientNoteCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Allergy => write!(f, "アレルギー歴"),
            Self::AdverseEvent => write!(f, "副作用歴"),
            Self::PastHistory => write!(f, "既往歴"),
            Self::Other => write!(f, "その他")
        }
    }
}

impl FromStr for SpecialPatientNoteCategory {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "アレルギー歴" | "アレルギー" | "allergy" => Ok(Self::Allergy),
            "2" | "副作用歴" | "副作用" | "adverse event" => Ok(Self::AdverseEvent),
            "3" | "既往歴" | "既往" | "past history" => Ok(Self::PastHistory),
            "9" | "その他" | "other" => Ok(Self::Other),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to SpecialPatientNoteCategory, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for SpecialPatientNoteCategory {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::Allergy),
            2 => Ok(Self::AdverseEvent),
            3 => Ok(Self::PastHistory),
            9 => Ok(Self::Other),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to SpecialPatientNoteCategory, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DrugCodeType {
    None, // コードなし
    Receipt, // レセプト電算コード
    Mhlw, // 厚労省コード
    Yj, // YJコード
    Hot, // HOTコード
}

impl DrugCodeType {
    pub fn to_code(&self) -> String {
        match *self {
            Self::None => format!("1"),
            Self::Receipt => format!("2"),
            Self::Mhlw => format!("3"),
            Self::Yj => format!("4"),
            Self::Hot => format!("6"),
        }
    }
}

impl fmt::Display for DrugCodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::None => write!(f, "コードなし"),
            Self::Receipt => write!(f, "レセプト電算コード"),
            Self::Mhlw => write!(f, "厚労省コード"),
            Self::Yj => write!(f, "YJコード"),
            Self::Hot => write!(f, "HOTコード"),
        }
    }
}

impl FromStr for DrugCodeType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "コードなし" | "none" => Ok(Self::None),
            "2" | "レセプト電算コード" | "レセプト" | "receipt" => Ok(Self::Receipt),
            "3" | "厚労省コード" | "厚生労働省コード" | "厚労省" | "厚生労働省" | "MHLW" => Ok(Self::Mhlw),
            "4" | "YJコード" | "YJ" => Ok(Self::Yj),
            "6" | "HOTコード" | "HOT" => Ok(Self::Hot),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to DrugCodeType, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for DrugCodeType {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::None),
            2 => Ok(Self::Receipt),
            3 => Ok(Self::Mhlw),
            4 => Ok(Self::Yj),
            6 => Ok(Self::Hot),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to DrugCodeType, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsageCodeType {
    None, // コードなし
    Jami, // JAMI用法コード
}

impl UsageCodeType {
    pub fn to_code(&self) -> String {
        match *self {
            Self::None => format!("1"),
            Self::Jami => format!("2"),
        }
    }
}

impl fmt::Display for UsageCodeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::None => write!(f, "コードなし"),
            Self::Jami => write!(f, "JAMI用法コード"),
        }
    }
}

impl FromStr for UsageCodeType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" | "コードなし" | "none" => Ok(Self::None),
            "2" | "JAMI用法コード" | "JAMIコード" | "JAMI" => Ok(Self::Jami),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to UsageCodeType, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for UsageCodeType {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Self::None),
            2 => Ok(Self::Jami),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to UsageCodeType, got {}", n)
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProvidedInformationType {
    AdverseEventInHospital, // 30: 入院中に副作用が発現した薬剤に関する情報
    PostDischargeCare, // 31: 退院後の療養を担う保険医療機関での投薬又は
                       // 保険薬局での調剤に必要な服薬の状況
                       // 及び投薬上の工夫に関する情報
    Other, // 99: その他
}

impl ProvidedInformationType {
    pub fn to_code(&self) -> String {
        match *self {
            Self::AdverseEventInHospital => format!("30"),
            Self::PostDischargeCare => format!("31"),
            Self::Other => format!("99"),
        }
    }
}

impl fmt::Display for ProvidedInformationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::AdverseEventInHospital
                 => write!(f, "入院中に副作用が発現した薬剤に関する情報"),
            Self::PostDischargeCare
                => write!(f, "退院後の療養を担う保険医療機関での投薬又は保険薬局での調剤に\
                                必要な服薬の状況及び投薬上の工夫に関する情報"),
            Self::Other => write!(f, "その他"),
        }
    }
}

impl FromStr for ProvidedInformationType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "30" => Ok(Self::AdverseEventInHospital),
            "31" => Ok(Self::PostDischargeCare),
            "99" => Ok(Self::Other),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert str to ProvidedInformationType, got \"{}\"", s)
            )),
        }
    }
}

impl TryFrom<u32> for ProvidedInformationType {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self, Self::Error> {
        match n {
            30 => Ok(Self::AdverseEventInHospital),
            31 => Ok(Self::PostDischargeCare),
            99 => Ok(Self::Other),
            _ => Err(Error::InvalidArgument(
                format!("Cannot convert u32 to ProvidedInformationType, got \"{}\"", n)
            )),
        }
    }
}

pub trait Record {
    fn record_number(&self) -> u32;
    fn cols(&self) -> u32;
}

/// Version record (バージョンレコード)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionRecord {
    pub number: u32,
    pub output_category: OutputCategory, 
}

impl VersionRecord {
    pub fn new(number: u32, output_category: OutputCategory) -> Self {
        Self {number: number, output_category: output_category}
    }
    pub fn to_code(&self) -> String {
        format!("JAHISTC{:>02},{}", self.number, self.output_category.to_code())
    }
}

impl Default for VersionRecord {
    fn default() -> Self {
        Self {number: 6, output_category: OutputCategory::ToPatient}
    }
}

impl FromStr for VersionRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^JAHISTC(\d\d),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            return Ok(Self {
                number: (&cap[1]).parse().map_err(Error::ParseIntError)?,
                output_category: (&cap[2]).parse()?,
            })
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to VersionRecord, got \"{}\"", s)
        ))
    }
}

/// No. 1: Patient record (患者情報レコード)
#[derive(Debug, Clone, PartialEq)]
pub struct PatientRecord {
    pub name: String, // 患者氏名
    pub gender: Gender, // 患者性別
    pub day_of_birth: Date, // 患者生年月日
    pub zip_code: Option<String>, // 患者郵便番号
    pub address: Option<String>, // 患者住所
    pub telephone: Option<String>, // 患者電話番号
    pub emergency_contact_information: Option<String>, // 緊急連絡先
    pub blood_type: Option<String>, // 血液型
    pub body_weight: Option<f32>, // 体重
    pub name_in_kana: Option<String>, // 患者氏名カナ
}

impl PatientRecord {
    pub fn new(name: String, gender: Gender, day_of_birth: Date,
                zip_code: Option<String>, address: Option<String>,
                telephone: Option<String>, emergency_contact_information: Option<String>,
                blood_type: Option<String>, body_weight: Option<f32>, 
                name_in_kana: Option<String>) -> Self {
        Self {
            name: name,
            gender: gender,
            day_of_birth: day_of_birth,
            zip_code: zip_code,
            address: address,
            telephone: telephone,
            emergency_contact_information: emergency_contact_information,
            blood_type: blood_type,
            body_weight: body_weight,
            name_in_kana: name_in_kana,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{},{},{},{},{},{},{}",
            self.record_number().to_string(), // 1
            self.name,
            self.gender.to_code(),
            self.day_of_birth.to_code(),
            self.zip_code.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.address.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.telephone.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.emergency_contact_information.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.blood_type.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.body_weight.map(|v| v.to_string()).unwrap_or_default(),
            self.name_in_kana.as_ref().map(|s| s.clone()).unwrap_or_default()
        )
    }
}

impl Record for PatientRecord {
    fn record_number(&self) -> u32 {
        1
    }
    fn cols(&self) -> u32 {
        10
    }
}

impl Default for PatientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            gender: Gender::Male,
            day_of_birth: Date::Seireki{year: 1970, month: 1, day: 1},
            zip_code: None,
            address: None,
            telephone: None,
            emergency_contact_information: None,
            blood_type: None,
            body_weight: None,
            name_in_kana: None,
        }
    }
}

impl FromStr for PatientRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),(\d),(\d{8}|\w\d{6}),([^,]*),([^,]*),([^,]*),([^,]*),([^,]*),((?:[0-9]+(?:[.][0-9]*)?|[.][0-9]+)?),([^,]*)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "1" {
                return Ok(Self {
                    name: (&cap[2]).to_string(),
                    gender: (&cap[3]).parse()?,
                    day_of_birth: (&cap[4]).parse()?,
                    zip_code: if (&cap[5]).is_empty() {None} else {Some((&cap[5]).to_string())},
                    address: if (&cap[6]).is_empty() {None} else {Some((&cap[6]).to_string())},
                    telephone: if (&cap[7]).is_empty() {None} else {Some((&cap[7]).to_string())},
                    emergency_contact_information: if (&cap[8]).is_empty() {None} else {Some((&cap[8]).to_string())},
                    blood_type: if (&cap[9]).is_empty() {None} else {Some((&cap[9]).to_string())},
                    body_weight: if (&cap[10]).is_empty() {None} else {Some((&cap[10]).parse().map_err(Error::ParseFloatError)?)},
                    name_in_kana: if (&cap[11]).is_empty() {None} else {Some((&cap[11]).to_string())},
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to PatientRecord, got \"{}\"", s)
        ))
    }
}

/// No 2. Special patient note record (患者特記レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialPatientNoteRecord {
    pub category: SpecialPatientNoteCategory, // 患者特記種別
    pub content: String, // 患者特記内容
    pub created_by: RecordCreator, // レコード作成者
}

impl SpecialPatientNoteRecord {
    pub fn new(category: SpecialPatientNoteCategory,
                content: String, created_by: RecordCreator) -> Self {
        Self {
            category: category,
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 2
            self.category.to_code(),
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for SpecialPatientNoteRecord {
    fn record_number(&self) -> u32 {
        2
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for SpecialPatientNoteRecord {
    fn default() -> Self {
        Self {
            category: SpecialPatientNoteCategory::Other,
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for SpecialPatientNoteRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "2" {
                return Ok(Self {
                    category: (&cap[2]).parse()?,
                    content: (&cap[3]).to_string(),
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to SpecialPatientNoteRecord, got \"{}\"", s)
        ))
    }
}

/// No 3. OTC medicine record (一般用医薬品服用レコード )
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtcDrugRecord {
    pub drug_name: String, // 薬品名称
    pub start_date: Option<Date>, // 服用開始年月日
    pub end_date: Option<Date>, // 服用終了年月日
    pub created_by: RecordCreator, // レコード作成者
}

impl OtcDrugRecord {
    pub fn new(drug_name: String, start_date: Option<Date>,
                end_date: Option<Date>, created_by: RecordCreator) -> Self {
        Self {
            drug_name: drug_name,
            start_date: start_date,
            end_date: end_date,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{}",
            self.record_number().to_string(), // 3
            self.drug_name,
            self.start_date.map(|v| v.to_code()).unwrap_or_default(),
            self.end_date.map(|v| v.to_code()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for OtcDrugRecord {
    fn record_number(&self) -> u32 {
        3
    }
    fn cols(&self) -> u32 {
        4
    }
}

impl Default for OtcDrugRecord {
    fn default() -> Self {
        Self {
            drug_name: "".to_string(),
            start_date: None,
            end_date: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for OtcDrugRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),((?:\d{8}|\w\d{6})?),((?:\d{8}|\w\d{6})?),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "3" {
                return Ok(Self {
                    drug_name: (&cap[2]).to_string(),
                    start_date: if (&cap[3]).is_empty() {None} else {Some((&cap[3]).parse()?)},
                    end_date: if (&cap[4]).is_empty() {None} else {Some((&cap[4]).parse()?)},
                    created_by: (&cap[5]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to OtcDrugRecord, got \"{}\"", s)
        ))
    }
}

/// No 4. Memo record (手帳メモレコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoRecord {
    pub content: String, // 手帳メモ情報
    pub created_at: Option<Date>, // メモ入力年月日
    pub created_by: RecordCreator, // レコード作成者
}

impl MemoRecord {
    pub fn new(content: String, created_at: Option<Date>,
                created_by: RecordCreator) -> Self {
        Self {
            content: content,
            created_at: created_at,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 4
            self.content,
            self.created_at.map(|v| v.to_code()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for MemoRecord {
    fn record_number(&self) -> u32 {
        4
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for MemoRecord {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            created_at: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for MemoRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),((?:\d{8}|\w\d{6})?),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "4" {
                return Ok(Self {
                    content: (&cap[2]).to_string(),
                    created_at: if (&cap[3]).is_empty() {None} else {Some((&cap[3]).parse()?)},
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to MemoRecord, got \"{}\"", s)
        ))
    }
}

/// No 5. Date record (調剤等年月日レコード)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRecord {
    pub created_at: Date, // 調剤等年月日
    pub created_by: RecordCreator, // レコード作成者
}

impl DateRecord {
    pub fn new(created_at: Date, created_by: RecordCreator) -> Self {
        Self {created_at: created_at, created_by: created_by}
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{}",
            self.record_number().to_string(), // 5
            self.created_at.to_code(),
            self.created_by.to_code()
        )
    }
}

impl Record for DateRecord {
    fn record_number(&self) -> u32 {
        5
    }
    fn cols(&self) -> u32 {
        2
    }
}

impl Default for DateRecord {
    fn default() -> Self {
        Self {
            created_at: Date::Seireki{year: 1970, month: 1, day: 1},
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for DateRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d{8}|\w\d{6}),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "5" {
                return Ok(Self {
                    created_at: (&cap[2]).parse()?,
                    created_by: (&cap[3]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to DateRecord, got \"{}\"", s)
        ))
    }
}

/// No 11. Pharmacy record (調剤－医療機関等レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PharmacyRecord {
    pub name: String, // 医療機関等名称
    pub prefecture: Option<Prefecture>, // 医療機関等都道府県
    pub fee_table: Option<FeeTable>, // 医療機関等点数表
    pub institution_code: Option<String>, // 医療機関等コード
    pub zip_code: Option<String>, // 医療機関等郵便番号
    pub address: Option<String>, // 医療機関等住所
    pub telephone: Option<String>, // 医療機関等電話番号
    pub created_by: RecordCreator, // レコード作成者
}

impl PharmacyRecord {
    pub fn new(name: String, prefecture: Option<Prefecture>,
                fee_table: Option<FeeTable>, institution_code: Option<String>,
                zip_code: Option<String>, address: Option<String>,
                telephone: Option<String>, created_by: RecordCreator) -> Self {
        Self {
            name: name,
            prefecture: prefecture,
            fee_table: fee_table,
            institution_code: institution_code,
            zip_code: zip_code,
            address: address,
            telephone: telephone,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{},{},{},{},{}",
            self.record_number().to_string(), // 11
            self.name,
            self.prefecture.map(|v| v.to_code()).unwrap_or_default(),
            self.fee_table.map(|v| v.to_code()).unwrap_or_default(),
            self.institution_code.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.zip_code.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.address.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.telephone.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for PharmacyRecord {
    fn record_number(&self) -> u32 {
        11
    }
    fn cols(&self) -> u32 {
        8
    }
}

impl Default for PharmacyRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            prefecture: None,
            fee_table: None,
            institution_code: None,
            zip_code: None,
            address: None,
            telephone: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for PharmacyRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),(\d{0,2}),(\d?),([^,]*),([^,]*),([^,]*),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "11" {
                return Ok(Self {
                    name: (&cap[2]).to_string(),
                    prefecture: if (&cap[3]).is_empty() {None} else {Some((&cap[3]).parse()?)},
                    fee_table: if (&cap[4]).is_empty() {None} else {Some((&cap[4]).parse()?)},
                    institution_code: if (&cap[5]).is_empty() {None} else {Some((&cap[5]).to_string())},
                    zip_code: if (&cap[6]).is_empty() {None} else {Some((&cap[6]).to_string())},
                    address: if (&cap[7]).is_empty() {None} else {Some((&cap[7]).to_string())},
                    telephone: if (&cap[8]).is_empty() {None} else {Some((&cap[8]).to_string())},
                    created_by: (&cap[9]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to PharmacyRecord, got \"{}\"", s)
        ))
    }
}

/// No 15. Pharmacist record (調剤－医師・薬剤師レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PharmacistRecord {
    pub name: String, // 医師・薬剤師氏名
    pub contact_information: Option<String>, // 医師・薬剤師連絡先
    pub created_by: RecordCreator, // レコード作成者
}

impl PharmacistRecord {
    pub fn new(name: String, contact_information: Option<String>,
                created_by: RecordCreator) -> Self {
        Self {
            name: name,
            contact_information: contact_information,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 15
            self.name,
            self.contact_information.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for PharmacistRecord {
    fn record_number(&self) -> u32 {
        15
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for PharmacistRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            contact_information: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for PharmacistRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "15" {
                return Ok(Self {
                    name: (&cap[2]).to_string(),
                    contact_information: if (&cap[3]).is_empty() {None} else {Some((&cap[3]).to_string())},
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to PharmacistRecord, got \"{}\"", s)
        ))
    }
}

/// No 51. Medical institution record (処方－医療機関レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MedicalInstitutionRecord {
    pub name: String, // 医療機関名称
    pub prefecture: Option<Prefecture>, // 医療機関都道府県
    pub fee_table: Option<FeeTable>, // 医療機関点数表
    pub institution_code: Option<String>, // 医療機関コード
    pub created_by: RecordCreator, // レコード作成者
}

impl MedicalInstitutionRecord {
    pub fn new(name: String, prefecture: Option<Prefecture>,
                fee_table: Option<FeeTable>, institution_code: Option<String>,
                created_by: RecordCreator) -> Self {
        Self {
            name: name,
            prefecture: prefecture,
            fee_table: fee_table,
            institution_code: institution_code,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{},{}",
            self.record_number().to_string(), // 51
            self.name,
            self.prefecture.map(|v| v.to_code()).unwrap_or_default(),
            self.fee_table.map(|v| v.to_code()).unwrap_or_default(),
            self.institution_code.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for MedicalInstitutionRecord {
    fn record_number(&self) -> u32 {
        51
    }
    fn cols(&self) -> u32 {
        5
    }
}

impl Default for MedicalInstitutionRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            prefecture: None,
            fee_table: None,
            institution_code: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for MedicalInstitutionRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),(\d{0,2}),(\d?),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "51" {
                return Ok(Self {
                    name: (&cap[2]).to_string(),
                    prefecture: if (&cap[3]).is_empty() {None} else {Some((&cap[3]).parse()?)},
                    fee_table: if (&cap[4]).is_empty() {None} else {Some((&cap[4]).parse()?)},
                    institution_code: if (&cap[5]).is_empty() {None} else {Some((&cap[5]).to_string())},
                    created_by: (&cap[6]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to MedicalInstitutionRecord, got \"{}\"", s)
        ))
    }
}

/// No 55. Physician record (処方－医師レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicianRecord {
    pub name: String, // 医師氏名
    pub specialty: Option<String>, // 診療科名
    pub created_by: RecordCreator, // レコード作成者
}

impl PhysicianRecord {
    pub fn new(name: String, specialty: Option<String>,
                created_by: RecordCreator) -> Self {
        Self {
            name: name,
            specialty: specialty,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 55
            self.name,
            self.specialty.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for PhysicianRecord {
    fn record_number(&self) -> u32 {
        55
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for PhysicianRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            specialty: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for PhysicianRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "55" {
                return Ok(Self {
                    name: (&cap[2]).to_string(),
                    specialty: if (&cap[3]).is_empty() {None} else {Some((&cap[3]).to_string())},
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to PhysicianRecord, got \"{}\"", s)
        ))
    }
}

/// No 201. Drug record (薬品レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrugRecord {
    pub rp_number: u32, // RP番号
    pub name: String, // 薬品名称
    pub dosage: String, // 用量
    pub unit: String, // 単位名
    pub drug_code_type: DrugCodeType, // 薬品コード種別
    pub drug_code: Option<String>, // 薬品コード
    pub created_by: RecordCreator, // レコード作成者
}

impl DrugRecord {
    pub fn new(rp_number: u32, name: String, dosage: String, unit: String,
                drug_code_type: DrugCodeType, drug_code: Option<String>,
                created_by: RecordCreator) -> Self {
        Self {
            rp_number: rp_number,
            name: name, 
            dosage: dosage,
            unit: unit,
            drug_code_type: drug_code_type,
            drug_code: drug_code,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{},{},{},{}",
            self.record_number().to_string(), // 201
            self.rp_number,
            self.name,
            self.dosage,
            self.unit,
            self.drug_code_type.to_code(),
            self.drug_code.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for DrugRecord {
    fn record_number(&self) -> u32 {
        201
    }
    fn cols(&self) -> u32 {
        7
    }
}

impl Default for DrugRecord {
    fn default() -> Self {
        Self {
            rp_number: 1,
            name: "".to_string(),
            dosage: "".to_string(),
            unit: "".to_string(),
            drug_code_type: DrugCodeType::None,
            drug_code: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for DrugRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d+),([^,]*),([^,]*),([^,]*),(\d?),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "201" {
                return Ok(Self {
                    rp_number: (&cap[2]).parse().map_err(Error::ParseIntError)?,
                    name: (&cap[3]).to_string(),
                    dosage: (&cap[4]).to_string(),
                    unit: (&cap[5]).to_string(),
                    drug_code_type: (&cap[6]).parse()?,
                    drug_code: if (&cap[7]).is_empty() {None} else {Some((&cap[7]).to_string())},
                    created_by: (&cap[8]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to DrugRecord, got \"{}\"", s)
        ))
    }
}

/// No 281. Drug supplementary record (薬品補足レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrugSupplementaryRecord {
    pub rp_number: u32, // RP番号
    pub content: String, // 薬品補足情報
    pub created_by: RecordCreator, // レコード作成者
}

impl DrugSupplementaryRecord {
    pub fn new(rp_number: u32, content: String, created_by: RecordCreator) -> Self {
        Self {
            rp_number: rp_number,
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 281
            self.rp_number,
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for DrugSupplementaryRecord {
    fn record_number(&self) -> u32 {
        281
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for DrugSupplementaryRecord {
    fn default() -> Self {
        Self {
            rp_number: 1,
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for DrugSupplementaryRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d+),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "281" {
                return Ok(Self {
                    rp_number: (&cap[2]).parse().map_err(Error::ParseIntError)?,
                    content: (&cap[3]).to_string(),
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to DrugSupplementaryRecord, got \"{}\"", s)
        ))
    }
}

/// No 291. Drug notice record (薬品服用注意レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrugNoticeRecord {
    pub rp_number: u32, // RP番号
    pub content: String, // 内容
    pub created_by: RecordCreator, // レコード作成者
}

impl DrugNoticeRecord {
    pub fn new(rp_number: u32, content: String, created_by: RecordCreator) -> Self {
        Self {
            rp_number: rp_number,
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 291
            self.rp_number,
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for DrugNoticeRecord {
    fn record_number(&self) -> u32 {
        291
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for DrugNoticeRecord {
    fn default() -> Self {
        Self {
            rp_number: 1,
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for DrugNoticeRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d+),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "291" {
                return Ok(Self {
                    rp_number: (&cap[2]).parse().map_err(Error::ParseIntError)?,
                    content: (&cap[3]).to_string(),
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to DrugNoticeRecord, got \"{}\"", s)
        ))
    }
}

/// No 301. Usage record (用法レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageRecord {
    pub rp_number: u32, // RP番号
    pub name: String, // 用法名称
    pub quantity: Option<u32>, // 調剤数量
    pub unit: Option<String>, // 調剤単位
    pub dosage_form: Option<DosageForm>, // 剤型コード
    pub usage_code_type: Option<UsageCodeType>, // 用法コード種別
    pub usage_code: Option<String>, // 用法コード
    pub created_by: RecordCreator, // レコード作成者
}

impl UsageRecord {
    pub fn new(rp_number: u32, name: String, quantity: Option<u32>,
            unit: Option<String>, dosage_form: Option<DosageForm>,
            usage_code_type: Option<UsageCodeType>,
            usage_code: Option<String>, created_by: RecordCreator) -> Self {
        Self {
            rp_number: rp_number,
            name: name,
            quantity: quantity,
            unit: unit,
            dosage_form: dosage_form,
            usage_code_type: usage_code_type,
            usage_code: usage_code,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{},{},{},{},{}",
            self.record_number().to_string(), // 301
            self.rp_number,
            self.name,
            self.quantity.map(|v| v.to_string()).unwrap_or_default(),
            self.unit.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.dosage_form.map(|v| v.to_code()).unwrap_or_default(),
            self.usage_code_type.map(|v| v.to_code()).unwrap_or_default(),
            self.usage_code.as_ref().map(|s| s.clone()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for UsageRecord {
    fn record_number(&self) -> u32 {
        301
    }
    fn cols(&self) -> u32 {
        8
    }
}

impl Default for UsageRecord {
    fn default() -> Self {
        Self {
            rp_number: 1,
            name: "".to_string(),
            quantity: None,
            unit: None,
            dosage_form: None,
            usage_code_type: None,
            usage_code: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for UsageRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d+),([^,]*),(\d*),([^,]*),(\d*),(\d?),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "301" {
                return Ok(Self {
                    rp_number: (&cap[2]).parse().map_err(Error::ParseIntError)?,
                    name: (&cap[3]).to_string(),
                    quantity: if (&cap[4]).is_empty() {None}
                        else {Some((&cap[4]).parse().map_err(Error::ParseIntError)?)},
                    unit: if (&cap[5]).is_empty() {None} else {Some((&cap[5]).to_string())},
                    dosage_form: if (&cap[6]).is_empty() {None} else {Some((&cap[6]).parse()?)},
                    usage_code_type: if (&cap[7]).is_empty() {None} else {Some((&cap[7]).parse()?)},
                    usage_code: if (&cap[8]).is_empty() {None} else {Some((&cap[8]).to_string())},
                    created_by: (&cap[9]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to UsageRecord, got \"{}\"", s)
        ))
    }
}

/// No 311. Usage supplementary record (用法補足レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageSupplementaryRecord {
    pub rp_number: u32, // RP番号
    pub content: String, // 用法補足情報
    pub created_by: RecordCreator, // レコード作成者
}

impl UsageSupplementaryRecord {
    pub fn new(rp_number: u32, content: String, created_by: RecordCreator) -> Self {
        Self {
            rp_number: rp_number,
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 311
            self.rp_number,
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for UsageSupplementaryRecord {
    fn record_number(&self) -> u32 {
        311
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for UsageSupplementaryRecord {
    fn default() -> Self {
        Self {
            rp_number: 1,
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for UsageSupplementaryRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d+),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "311" {
                return Ok(Self {
                    rp_number: (&cap[2]).parse().map_err(Error::ParseIntError)?,
                    content: (&cap[3]).to_string(),
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to UsageSupplementaryRecord, got \"{}\"", s)
        ))
    }
}

/// No 391. Rp notice record (処方服用注意レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RpNoticeRecord {
    pub rp_number: u32, // RP番号
    pub content: String, // 内容
    pub created_by: RecordCreator, // レコード作成者
}

impl RpNoticeRecord {
    pub fn new(rp_number: u32, content: String, created_by: RecordCreator) -> Self {
        Self {
            rp_number: rp_number,
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 391
            self.rp_number,
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for RpNoticeRecord {
    fn record_number(&self) -> u32 {
        391
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for RpNoticeRecord {
    fn default() -> Self {
        Self {
            rp_number: 1,
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for RpNoticeRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),(\d+),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "391" {
                return Ok(Self {
                    rp_number: (&cap[2]).parse().map_err(Error::ParseIntError)?,
                    content: (&cap[3]).to_string(),
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to RpNoticeRecord, got \"{}\"", s)
        ))
    }
}

/// No 401. Notice record (服用注意レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeRecord {
    pub content: String, // 内容
    pub created_by: RecordCreator, // レコード作成者
}

impl NoticeRecord {
    pub fn new(content: String, created_by: RecordCreator) -> Self {
        Self {
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{}",
            self.record_number().to_string(), // 401
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for NoticeRecord {
    fn record_number(&self) -> u32 {
        401
    }
    fn cols(&self) -> u32 {
        2
    }
}

impl Default for NoticeRecord {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for NoticeRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "401" {
                return Ok(Self {
                    content: (&cap[2]).to_string(),
                    created_by: (&cap[3]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to NoticeRecord, got \"{}\"", s)
        ))
    }
}

/// No 411. Information provision record (医療機関等提供情報レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InformationProvisionRecord {
    pub content: String, // 内容
    pub information_type: ProvidedInformationType, // 提供情報種別
    pub created_by: RecordCreator, // レコード作成者
}

impl InformationProvisionRecord {
    pub fn new(content: String, information_type: ProvidedInformationType,
                created_by: RecordCreator) -> Self {
        Self {
            content: content,
            information_type: information_type,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{}",
            self.record_number().to_string(), // 411
            self.content,
            self.information_type.to_code(),
            self.created_by.to_code()
        )
    }
}

impl Record for InformationProvisionRecord {
    fn record_number(&self) -> u32 {
        411
    }
    fn cols(&self) -> u32 {
        3
    }
}

impl Default for InformationProvisionRecord {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            information_type: ProvidedInformationType::Other,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for InformationProvisionRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),(\d{1,2}),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "411" {
                return Ok(Self {
                    content: (&cap[2]).to_string(),
                    information_type: (&cap[3]).parse()?,
                    created_by: (&cap[4]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to InformationProvisionRecord, got \"{}\"", s)
        ))
    }
}

/// No 501. Note record (備考レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteRecord {
    pub content: String, // 備考情報
    pub created_by: RecordCreator, // レコード作成者
}

impl NoteRecord {
    pub fn new(content: String, created_by: RecordCreator) -> Self {
        Self {
            content: content,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{}",
            self.record_number().to_string(), // 501
            self.content,
            self.created_by.to_code()
        )
    }
}

impl Record for NoteRecord {
    fn record_number(&self) -> u32 {
        501
    }
    fn cols(&self) -> u32 {
        2
    }
}

impl Default for NoteRecord {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for NoteRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "501" {
                return Ok(Self {
                    content: (&cap[2]).to_string(),
                    created_by: (&cap[3]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to NoteRecord, got \"{}\"", s)
        ))
    }
}

/// No 601. From patient record (患者等記入レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FromPatientRecord {
    pub content: String, // 患者等記入情報
    pub created_at: Option<Date>, // 入力年月日
}

impl FromPatientRecord {
    pub fn new(content: String, created_at: Option<Date>) -> Self {
        Self {
            content: content,
            created_at: created_at,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{}",
            self.record_number().to_string(), // 601
            self.content,
            self.created_at.map(|v| v.to_code()).unwrap_or_default(),
        )
    }
}

impl Record for FromPatientRecord {
    fn record_number(&self) -> u32 {
        601
    }
    fn cols(&self) -> u32 {
        2
    }
}

impl Default for FromPatientRecord {
    fn default() -> Self {
        Self {
            content: "".to_string(),
            created_at: None,
        }
    }
}

impl FromStr for FromPatientRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),((?:\d{8}|\w\d{6})?)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "601" {
                return Ok(Self {
                    content: (&cap[2]).to_string(),
                    created_at: if (&cap[2]).is_empty() {None} else {Some((&cap[3]).parse()?)},
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to FromPatientRecord, got \"{}\"", s)
        ))
    }
}

/// No 701. Family pharmacist record (かかりつけ薬剤師レコード)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyPharmacistRecord {
    pub name: String, // かかりつけ薬剤師氏名
    pub pharmacy_name: String, // 勤務先薬局名称
    pub contact_information: String, // 連絡先
    pub start_date: Option<Date>, // 担当開始年月日
    pub end_date: Option<Date>, // 担当終了年月日
    pub created_by: RecordCreator, // レコード作成者
}

impl FamilyPharmacistRecord {
    pub fn new(name: String, pharmacy_name: String,
                contact_information: String,
                start_date: Option<Date>,
                end_date: Option<Date>,
                created_by: RecordCreator) -> Self {
        Self {
            name: name,
            pharmacy_name: pharmacy_name,
            contact_information: contact_information,
            start_date: start_date,
            end_date: end_date,
            created_by: created_by,
        }
    }

    pub fn to_code(&self) -> String {
        format!("{},{},{},{},{},{},{}",
            self.record_number().to_string(), // 701
            self.name,
            self.pharmacy_name,
            self.contact_information,
            self.start_date.map(|v| v.to_code()).unwrap_or_default(),
            self.end_date.map(|v| v.to_code()).unwrap_or_default(),
            self.created_by.to_code()
        )
    }
}

impl Record for FamilyPharmacistRecord {
    fn record_number(&self) -> u32 {
        701
    }
    fn cols(&self) -> u32 {
        6
    }
}

impl Default for FamilyPharmacistRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            pharmacy_name: "".to_string(),
            contact_information: "".to_string(),
            start_date: None,
            end_date: None,
            created_by: RecordCreator::Unknown,
        }
    }
}

impl FromStr for FamilyPharmacistRecord {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(\d+),([^,]*),([^,]*),([^,]*),((?:\d{8}|\w\d{6})?),((?:\d{8}|\w\d{6})?),(\d)$").unwrap();
        }
        for cap in RE.captures_iter(s) {
            if (&cap[1]) == "701" {
                return Ok(Self {
                    name: (&cap[2]).to_string(),
                    pharmacy_name: (&cap[3]).to_string(),
                    contact_information: (&cap[4]).to_string(),
                    start_date: if (&cap[5]).is_empty() {None} else {Some((&cap[5]).parse()?)},
                    end_date: if (&cap[6]).is_empty() {None} else {Some((&cap[6]).parse()?)},
                    created_by: (&cap[7]).parse()?,
                })
            }
        }
        Err(Error::InvalidRecordLine(
            format!("Cannot convert str to FamilyPharmacistRecord, got \"{}\"", s)
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrugBlock {
    pub drug: DrugRecord, // 薬品レコード
    pub drug_supplementary: Vec<DrugSupplementaryRecord>, // 薬品補足レコード
    pub drug_notice: Vec<DrugNoticeRecord>, // 薬品服用注意レコード
}

impl DrugBlock {
    pub fn to_code(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(self.drug.to_code());
        for record in &self.drug_supplementary {
            lines.push(record.to_code());
        }
        for record in &self.drug_notice {
            lines.push(record.to_code());
        }
        lines.join("\r\n")
    }
}

impl Default for DrugBlock {
    fn default() -> Self {
        Self {
            drug: DrugRecord::default(),
            drug_supplementary: Vec::new(),
            drug_notice: Vec::new(),
        }
    }
}

impl FromStr for DrugBlock {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut drug: Option<DrugRecord> = None;
        let mut drug_supplementary: Vec<DrugSupplementaryRecord> = Vec::new();
        let mut drug_notice: Vec<DrugNoticeRecord> = Vec::new();
        for line in s.to_string().lines() {
            if line.chars().count() >= 4 {
                let sep = line.char_indices().nth(4).unwrap().0;
                if drug.is_none() {
                    if &line[..sep] == "201," { // 薬品レコード
                        drug = Some(line.parse()?);
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Expected 201 DrugRecord, got \"{}\".", line)
                            )
                        );
                    }
                } else {
                    if &line[..sep] == "281," { // 薬品補足レコード
                        drug_supplementary.push(line.parse()?);
                    } else if &line[..sep] == "291," { // 薬品服用注意レコード
                        drug_notice.push(line.parse()?);
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Expected 281 or 291 record, got \"{}\".", line)
                            )
                        );
                    }
                }
            } else if line == "" {
                continue
            } else {
                return Err(
                    Error::GotUnexpectedRecordLine(
                        format!("Expected 201, 281 or 291 record, got \"{}\".", line)
                    )
                );
            }
        }
        if drug.is_some() {
            Ok(Self {
                drug: drug.unwrap(),
                drug_supplementary: drug_supplementary,
                drug_notice: drug_notice,
            })
        } else {
            Err(Error::MissingRequiredRecord(format!("DrugRecord is required.")))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RpBlock {
    pub drugs: Vec<DrugBlock>, // 薬品情報
    pub usage: UsageRecord, // 用法レコード
    pub usage_supplementary: Vec<UsageSupplementaryRecord>, // 用法補足レコード
    pub rp_notice: Vec<RpNoticeRecord>, // 処方服用注意レコード
}

impl RpBlock {
    pub fn to_code(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        for block in &self.drugs {
            lines.push(block.to_code());
        }
        lines.push(self.usage.to_code());
        for record in &self.usage_supplementary {
            lines.push(record.to_code());
        }
        for record in &self.rp_notice {
            lines.push(record.to_code());
        }
        lines.join("\r\n")
    }
}

impl Default for RpBlock {
    fn default() -> Self {
        Self {
            drugs: Vec::new(),
            usage: UsageRecord::default(),
            usage_supplementary: Vec::new(),
            rp_notice: Vec::new(),
        }
    }
}

impl FromStr for RpBlock {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut drug_blocks: Vec<DrugBlock> = Vec::new();
        let mut temp_drug_block_string: Vec<String> = Vec::new();
        let mut usage: Option<UsageRecord> = None;
        let mut usage_supplementary: Vec<UsageSupplementaryRecord> = Vec::new();
        let mut rp_notice: Vec<RpNoticeRecord> = Vec::new();
        for line in s.to_string().lines() {
            if line.chars().count() >= 4 {
                let sep = line.char_indices().nth(4).unwrap().0;
                if usage.is_none() {
                    if &line[..sep] == "201," { // 薬品レコード
                        if temp_drug_block_string.len() > 0 {
                            drug_blocks.push(temp_drug_block_string.join("\r\n").parse()?);
                            temp_drug_block_string = Vec::new();
                        }
                        temp_drug_block_string.push(line.to_string());
                    } else if &line[..sep] == "281," || &line[..sep] == "291," { // 薬品補足 薬品服用注意レコード
                        if temp_drug_block_string.len() == 0 {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("201 DrugRecord line must exist before \"{}\"", line)
                                )
                            );
                        }
                        temp_drug_block_string.push(line.to_string());
                    } else if &line[..sep] == "301," { // 用法レコード
                        if temp_drug_block_string.len() > 0 {
                            drug_blocks.push(temp_drug_block_string.join("\r\n").parse()?);
                        } else if drug_blocks.len() == 0 {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("DrugBlock must exist before UsageRecord \"{}\"", line)
                                )
                            );
                        }
                        usage = Some(line.parse()?);
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Expected DrugBlock or 301 UsageRecord, got \"{}\".", line)
                            )
                        );
                    }
                } else {
                    if &line[..sep] == "311," { // 用法補足レコード
                        usage_supplementary.push(line.parse()?);
                    } else if &line[..sep] == "391," { // 処方服用注意レコード
                        rp_notice.push(line.parse()?);
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Expected 311 or 391 record, got \"{}\".", line)
                            )
                        );
                    }
                }
            } else if line == "" {
                continue
            } else {
                return Err(
                    Error::GotUnexpectedRecordLine(
                        format!("Expected 201, 281, 291, 301, 311, or 391 record, got \"{}\".", line)
                    )
                );
            }
        }
        if drug_blocks.len() > 0 && usage.is_some() {
            Ok(Self {
                drugs: drug_blocks,
                usage: usage.unwrap(),
                usage_supplementary: usage_supplementary,
                rp_notice: rp_notice,
            })
        } else {
            if drug_blocks.len() == 0 {
                Err(Error::MissingRequiredRecord(format!("DrugBlock is required.")))
            } else {
                Err(Error::MissingRequiredRecord(format!("UsageRecord is required.")))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrescriptionBlock {
    pub physician: Option<PhysicianRecord>, // 処方－医師レコード
    pub rps: Vec<RpBlock>, // RP情報
}

impl PrescriptionBlock {
    pub fn to_code(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        if let Some(record) = &self.physician {
            lines.push(record.to_code());
        }
        for block in &self.rps {
            lines.push(block.to_code());
        }
        lines.join("\r\n")
    }
}

impl Default for PrescriptionBlock {
    fn default() -> Self {
        Self {
            physician: None,
            rps: Vec::new(),
        }
    }
}

impl FromStr for PrescriptionBlock {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut physician: Option<PhysicianRecord> = None;
        let mut rps: Vec<RpBlock> = Vec::new();
        let mut temp_rp_block_string: Vec<String> = Vec::new();
        let mut flag_usage_exists: bool = false;
        for line in s.to_string().lines() {
            if line.chars().count() >= 4 {
                let sep3 = line.char_indices().nth(3).unwrap().0;
                let sep4 = line.char_indices().nth(4).unwrap().0;
                if &line[..sep3] == "55," { // 医師レコード
                    if physician.is_some() {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Multiple PhysicianRecord lines are not allowed: \"{}\"", line)
                            )
                        );
                    } else if rps.len() > 0 || temp_rp_block_string.len() > 0 {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("PhysicianRecord must exist before RpBlock")
                            )
                        );
                    } else {
                        physician = Some(line.parse()?);
                    }
                } else if &line[..sep4] == "201," { // 薬品レコード
                    if flag_usage_exists && temp_rp_block_string.len() > 0  {
                        rps.push(temp_rp_block_string.join("\r\n").parse()?);
                        temp_rp_block_string = Vec::new();
                        flag_usage_exists = false;
                    }
                    temp_rp_block_string.push(line.to_string());
                } else if &line[..sep4] == "301," { // 用法レコード
                    flag_usage_exists = true;
                    temp_rp_block_string.push(line.to_string());
                } else if &line[..sep4] == "281," || &line[..sep4] == "291,"
                        || &line[..sep4] == "311," || &line[..sep4] == "391," {
                    temp_rp_block_string.push(line.to_string());
                } else {
                    return Err(
                        Error::GotUnexpectedRecordLine(
                            format!("Expected 311 or 391 record, got \"{}\".", line)
                        )
                    );
                }
            } else if line == "" {
                continue
            } else {
                return Err(
                    Error::GotUnexpectedRecordLine(
                        format!("Expected 55 record or RpBlock, got \"{}\".", line)
                    )
                );
            }
        }
        if temp_rp_block_string.len() > 0  {
            rps.push(temp_rp_block_string.join("\r\n").parse()?);
        }
        Ok(Self {
            physician: physician,
            rps: rps,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispensingInformationBlock {
    pub date: DateRecord, // 調剤等年月日レコード
    pub pharmacy: PharmacyRecord, // 調剤－医療機関等レコード
    pub pharmacist: Option<PharmacistRecord>, // 調剤－医師・薬剤師レコード
    pub medical_institute: Option<MedicalInstitutionRecord>, // 処方－医療機関レコード

    pub prescriptions: Vec<PrescriptionBlock>,  // 処方

    pub notice: Option<NoticeRecord>, // 服用注意レコード
    pub information_provision: Option<InformationProvisionRecord>, //医療機関等提供情報レコード
    pub note: Option<NoteRecord>, // 備考レコード
    pub from_patient: Option<FromPatientRecord>, // 患者等記入レコード
}

impl DispensingInformationBlock {
    pub fn to_code(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(self.date.to_code());
        lines.push(self.pharmacy.to_code());
        if let Some(record) = &self.pharmacist {
            lines.push(record.to_code());
        }
        if let Some(record) = &self.medical_institute {
            lines.push(record.to_code());
        }
        for block in &self.prescriptions {
            lines.push(block.to_code());
        }
        if let Some(record) = &self.notice {
            lines.push(record.to_code());
        }
        if let Some(record) = &self.information_provision {
            lines.push(record.to_code());
        }
        if let Some(record) = &self.note {
            lines.push(record.to_code());
        }
        if let Some(record) = &self.from_patient {
            lines.push(record.to_code());
        }
        lines.join("\r\n")
    }
}

impl Default for DispensingInformationBlock {
    fn default() -> Self {
        Self {
            date: DateRecord::default(),
            pharmacy: PharmacyRecord::default(),
            pharmacist: None,
            medical_institute: None,

            prescriptions: Vec::new(),

            notice: None,
            information_provision: None,
            note: None,
            from_patient: None,
        }
    }
}

impl FromStr for DispensingInformationBlock {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut date: Option<DateRecord> = None;
        let mut pharmacy: Option<PharmacyRecord> = None;
        let mut pharmacist: Option<PharmacistRecord> = None;
        let mut medical_institute: Option<MedicalInstitutionRecord> = None;

        let mut prescriptions: Vec<PrescriptionBlock> = Vec::new();
        let mut temp_prescription_block_string: Vec<String> = Vec::new();

        let mut notice: Option<NoticeRecord> = None;
        let mut information_provision: Option<InformationProvisionRecord> = None;
        let mut note: Option<NoteRecord> = None;
        let mut from_patient: Option<FromPatientRecord> = None;

        let mut cur_num: u32 = 0;

        for line in s.to_string().lines() {
            if line.chars().count() >= 4 {
                let sep2 = line.char_indices().nth(2).unwrap().0;
                let sep3 = line.char_indices().nth(3).unwrap().0;
                let sep4 = line.char_indices().nth(4).unwrap().0;
                if &line[..sep2] == "5," { // 調剤等年月日レコード
                    if date.is_none() {
                        date = Some(line.parse()?);
                        cur_num = 5;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Multiple DateRecord lines are not allowed: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep3] == "11," { // 調剤－医療機関等レコード
                    if cur_num < 11 {
                        if pharmacy.is_none() {
                            pharmacy = Some(line.parse()?);
                            cur_num = 11;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple PharmacyRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected PharmacyRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep3] == "15," { // 調剤－医師・薬剤師レコード
                    if cur_num < 15 {
                        if pharmacist.is_none() {
                            pharmacist = Some(line.parse()?);
                            cur_num = 15;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple PharmacistRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected PharmacistRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep3] == "51," { // 処方－医療機関レコード
                    if cur_num < 51 {
                        if medical_institute.is_none() {
                            medical_institute = Some(line.parse()?);
                            cur_num = 51;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple MedicalInstituteRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected MedicalInstituteRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep3] == "55," || &line[..sep4] == "201,"
                        || &line[..sep4] == "281," || &line[..sep4] == "291,"
                        || &line[..sep4] == "301," || &line[..sep4] == "311," || &line[..sep4] == "391," {
                    if cur_num <= 55 {
                        if &line[..sep3] == "55," && temp_prescription_block_string.len() > 0 {
                            prescriptions.push(temp_prescription_block_string.join("\r\n").parse()?);
                            temp_prescription_block_string = Vec::new();
                        }
                        temp_prescription_block_string.push(line.to_string());
                        cur_num = 55;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected PrescriptionBlock here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep4] == "401," { // 服用注意レコード
                    if cur_num < 401 {
                        if notice.is_none() {
                            notice = Some(line.parse()?);
                            cur_num = 401;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple NoticeRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected NoticeRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep4] == "411," { // 医療機関等提供情報レコード
                    if cur_num < 411 {
                        if information_provision.is_none() {
                            information_provision = Some(line.parse()?);
                            cur_num = 411;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple InformationProvisionRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected InformationProvisionRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep4] == "501," { // 備考レコード
                    if cur_num < 501 {
                        if note.is_none() {
                            note = Some(line.parse()?);
                            cur_num = 501;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple NoteRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected NoteRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep4] == "601," { // 患者記入レコード
                    if cur_num < 601 {
                        if from_patient.is_none() {
                            from_patient = Some(line.parse()?);
                            cur_num = 601;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple FromPatientRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected FromPatientRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else {
                    return Err(
                        Error::GotUnexpectedRecordLine(
                            format!("Expected 5, 11, 15, 51, 55, 201~391, 401, 411, 501, or 601 record, got \"{}\".", line)
                        )
                    );
                }
            } else if line == "" {
                continue
            } else {
                return Err(
                    Error::GotUnexpectedRecordLine(
                        format!("Expected record line of DispensingInformationBlock, got \"{}\".", line)
                    )
                );
            }
        }
        if temp_prescription_block_string.len() > 0  {
            prescriptions.push(temp_prescription_block_string.join("\r\n").parse()?);
        }
        if date.is_some() && pharmacy.is_some() {
            Ok(Self {
                date: date.unwrap(),
                pharmacy: pharmacy.unwrap(),
                pharmacist: pharmacist,
                medical_institute: medical_institute,
                prescriptions: prescriptions,
                notice: notice,
                information_provision: information_provision,
                note: note,
                from_patient: from_patient,
            })
        } else {
            if date.is_none() {
                Err(Error::MissingRequiredRecord(format!("DateRecord is required.")))
            } else {
                Err(Error::MissingRequiredRecord(format!("PharmacyRecord is required.")))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MedicineNotebook {
    pub version: VersionRecord, // バージョンレコード
    pub patient: PatientRecord, // 患者情報レコード
    pub special_patient_notes: Vec<SpecialPatientNoteRecord>, // 患者特記レコード
    pub otc_drugs: Vec<OtcDrugRecord>, // 一般用医薬品服用レコード
    pub memos: Vec<MemoRecord>, // 手帳メモレコード

    pub dispensing_information: Vec<DispensingInformationBlock>, // 調剤情報

    pub family_pharmacist: Vec<FamilyPharmacistRecord>, // かかりつけ薬剤師レコード
}

impl MedicineNotebook {
    pub fn to_code(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(self.version.to_code());
        lines.push(self.patient.to_code());
        for record in &self.special_patient_notes {
            lines.push(record.to_code());
        }
        for record in &self.otc_drugs {
            lines.push(record.to_code());
        }
        for record in &self.memos {
            lines.push(record.to_code());
        }
        for block in &self.dispensing_information {
            lines.push(block.to_code());
        }
        for record in &self.family_pharmacist {
            lines.push(record.to_code());
        }
        lines.join("\r\n")
    }
}

impl Default for MedicineNotebook {
    fn default() -> Self {
        Self {
            version: VersionRecord::default(),
            patient: PatientRecord::default(),
            special_patient_notes: Vec::new(),
            otc_drugs: Vec::new(),
            memos: Vec::new(),

            dispensing_information: Vec::new(),

            family_pharmacist: Vec::new(),
        }
    }
}

impl FromStr for MedicineNotebook {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut version: Option<VersionRecord> = None;
        let mut patient: Option<PatientRecord> = None;
        let mut special_patient_notes: Vec<SpecialPatientNoteRecord> = Vec::new();
        let mut otc_drugs: Vec<OtcDrugRecord> = Vec::new();
        let mut memos: Vec<MemoRecord> = Vec::new();

        let mut dispensing_information: Vec<DispensingInformationBlock> = Vec::new();
        let mut temp_disp_info_block_string: Vec<String> = Vec::new();

        let mut family_pharmacist: Vec<FamilyPharmacistRecord> = Vec::new();

        let mut cur_num: u32 = 0;

        for line in s.to_string().lines() {
            if line.chars().count() >= 4 {
                let sep2 = line.char_indices().nth(2).unwrap().0;
                let sep3 = line.char_indices().nth(3).unwrap().0;
                let sep4 = line.char_indices().nth(4).unwrap().0;
                let sep7 = line.char_indices().nth(7).unwrap().0;
                if &line[..sep7] == "JAHISTC" { // バージョンレコード
                    if cur_num == 0 {
                        if version.is_none() {
                            version = Some(line.parse()?);
                            cur_num = 0;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple VersionRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected VersionRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep2] == "1," { // 患者情報レコード
                    if cur_num < 1 {
                        if patient.is_none() {
                            patient = Some(line.parse()?);
                            cur_num = 1;
                        } else {
                            return Err(
                                Error::GotUnexpectedRecordLine(
                                    format!("Multiple PatientRecord lines are not allowed: \"{}\"", line)
                                )
                            );
                        }
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected PatientRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep2] == "2," { // 患者特記レコード
                    if cur_num <= 2 {
                        special_patient_notes.push(line.parse()?);
                        cur_num = 2;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected SpecialPatientNoteRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep2] == "3," { // 一般用医薬品服用レコード
                    if cur_num <= 3 {
                        otc_drugs.push(line.parse()?);
                        cur_num = 3;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected OtcDrugRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep2] == "4," { // 手帳メモレコード
                    if cur_num <= 4 {
                        memos.push(line.parse()?);
                        cur_num = 4;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected MemoRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep2] == "5," || &line[..sep3] == "11,"
                        || &line[..sep3] == "15," || &line[..sep3] == "51,"
                        || &line[..sep3] == "55," || &line[..sep4] == "201,"
                        || &line[..sep4] == "281," || &line[..sep4] == "291,"
                        || &line[..sep4] == "301," || &line[..sep4] == "311,"
                        || &line[..sep4] == "391," || &line[..sep4] == "401,"
                        || &line[..sep4] == "411," || &line[..sep4] == "501," || &line[..sep4] == "601," {
                    if cur_num <= 5 {
                        if &line[..sep2] == "5," && temp_disp_info_block_string.len() > 0 {
                            dispensing_information.push(temp_disp_info_block_string.join("\r\n").parse()?);
                            temp_disp_info_block_string = Vec::new();
                        }
                        temp_disp_info_block_string.push(line.to_string());
                        cur_num = 5;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected DispensingInformationBlock here: \"{}\"", line)
                            )
                        );
                    }
                } else if &line[..sep4] == "701," { //かかりつけ薬剤師レコード
                    if cur_num <= 701 {
                        family_pharmacist.push(line.parse()?);
                        cur_num = 701;
                    } else {
                        return Err(
                            Error::GotUnexpectedRecordLine(
                                format!("Unexpected FamilyPharmacistRecord here: \"{}\"", line)
                            )
                        );
                    }
                } else {
                    return Err(
                        Error::GotUnexpectedRecordLine(
                            format!("Expected valid record line, got \"{}\".", line)
                        )
                    );
                }
            } else if line == "" {
                continue
            } else {
                return Err(
                    Error::GotUnexpectedRecordLine(
                        format!("Expected valid record line, got \"{}\".", line)
                    )
                );
            }
        }
        if temp_disp_info_block_string.len() > 0  {
            dispensing_information.push(temp_disp_info_block_string.join("\r\n").parse()?);
        }
        if version.is_some() && patient.is_some() {
            Ok(Self {
                version: version.unwrap(),
                patient: patient.unwrap(),
                special_patient_notes: special_patient_notes,
                otc_drugs: otc_drugs,
                memos: memos,

                dispensing_information: dispensing_information,

                family_pharmacist: family_pharmacist,
            })
        } else {
            if version.is_some() {
                Err(Error::MissingRequiredRecord(format!("VersionRecord is required.")))
            } else {
                Err(Error::MissingRequiredRecord(format!("PatientRecord is required.")))
            }
        }
    }
}


/*
/// Converts from a string slice of date in seireki to `chrono::NaiveDate`.
/// 
/// # Arguments
/// 
/// * `s` - A string slice that holds date as 'YYYYMMDD' form
/// 
/// # Examples
/// 
/// ```
/// use kartech::jahis::seireki8_to_naivedate;
/// let s = "20191102";
/// println!("{:?}", seireki8_to_naivedate(s)) // Ok(2019-11-02)
/// ```
pub fn seireki8_to_naivedate(s: &str) -> Result<chrono::NaiveDate, Error> {
    if s.chars().count() == 8 && s.chars().all(char::is_numeric){
        let y_m = s.char_indices().nth(4).unwrap().0; // https://qiita.com/7ma7X/items/7fb68395984958987a54
        let m_d = s.char_indices().nth(6).unwrap().0;
        let year: i32 = (&s[..y_m]).parse().map_err(Error::ParseIntError)?;
        let month: u32 = (&s[y_m..m_d]).parse().map_err(Error::ParseIntError)?;
        let day: u32 = (&s[m_d..]).parse().map_err(Error::ParseIntError)?;
        Ok(chrono::NaiveDate::from_ymd(year, month, day))
    } else {
        Err(Error::InvalidArgument)
    }
}

/// Converts from a string slice of date in wareki to `chrono::NaiveDate`.
/// 
/// # Arguments
/// 
/// * `s` - A string slice that holds date as 'GYYMMDD' form
/// 
/// # Examples
/// 
/// ```
/// use kartech::jahis::wareki7_to_naivedate;
/// let s = "R011102";
/// println!("{:?}", wareki7_to_naivedate(s)) // Ok(2019-11-02)
/// ```
pub fn wareki7_to_naivedate(s: &str) -> Result<chrono::NaiveDate, Error> {
    if s.chars().count() == 7 {
        let g_y = s.char_indices().nth(1).unwrap().0; // https://qiita.com/7ma7X/items/7fb68395984958987a54
        let y_m = s.char_indices().nth(3).unwrap().0;
        let m_d = s.char_indices().nth(5).unwrap().0;
        let gengo = &s[..g_y];
        let g_year: i32 = (&s[g_y..y_m]).parse().map_err(Error::ParseIntError)?;
        let month: u32 = (&s[y_m..m_d]).parse().map_err(Error::ParseIntError)?;
        let day: u32 = (&s[m_d..]).parse().map_err(Error::ParseIntError)?;
        match gengo {
            "R" => Ok(chrono::NaiveDate::from_ymd(g_year + 2018, month, day)),
            "H" => Ok(chrono::NaiveDate::from_ymd(g_year + 1988, month, day)),
            "S" => Ok(chrono::NaiveDate::from_ymd(g_year + 1925, month, day)),
            "T" => Ok(chrono::NaiveDate::from_ymd(g_year + 1911, month, day)),
            "M" => Ok(chrono::NaiveDate::from_ymd(g_year + 1867, month, day)),
            _ => Err(Error::InvalidArgument)
        }
    } else {
        Err(Error::InvalidArgument)
    }
}


/// Converts from `chrono::NaiveDate` to `String` of date in seireki
/// 
/// # Arguments
/// 
/// * `d` - A `chrono::NaiveDate` to be converted to `String` of date in seireki
/// 
/// # Examples
/// 
/// ```
/// use chrono;
/// use kartech::jahis::naivedate_to_seireki8;
/// let d = chrono::NaiveDate::from_ymd(2019, 11, 2);
/// println!("{:?}", naivedate_to_seireki8(&d)) // Ok("20191102")
/// ```
pub fn naivedate_to_seireki8(d: &chrono::NaiveDate) -> Result<String, Error> {
    let year = d.year();
    let month = d.month();
    let day = d.day();
    Ok(format!("{:>04}{:>02}{:>02}", year, month, day))
}

/// Converts from `chrono::NaiveDate` to `String` of date in wareki
/// 
/// # Arguments
/// 
/// * `d` - A `chrono::NaiveDate` to be converted to `String` of date in wareki
/// 
/// # Examples
/// 
/// ```
/// use chrono;
/// use kartech::jahis::naivedate_to_wareki7;
/// let d = chrono::NaiveDate::from_ymd(2019, 11, 2);
/// println!("{:?}", naivedate_to_wareki7(&d)) // Ok("R011102")
/// ```
pub fn naivedate_to_wareki7(d: &chrono::NaiveDate) -> Result<String, Error> {
    let year = d.year();
    let month = d.month();
    let day = d.day();
    if year > 2019 || year == 2019 && month >= 5 {
        return Ok(format!("R{:>02}{:>02}{:>02}", year - 2018, month, day));
    } else if year > 1989 || year == 1989 && month > 1 || year == 1989 && month == 1 && day >= 8 {
        return Ok(format!("H{:>02}{:>02}{:>02}", year - 1988, month, day));
    } else if year > 1926 || year == 1926 && month == 12 && day >= 25 {
        return Ok(format!("S{:>02}{:>02}{:>02}", year - 1925, month, day));
    } else if year > 1912 || year == 1912 && month > 7 || year == 1912 && month == 7 && day >= 30 {
        return Ok(format!("T{:>02}{:>02}{:>02}", year - 1911, month, day));
    } else if year > 1872 {
        return Ok(format!("M{:>02}{:>02}{:>02}", year - 1867, month, day));
    } else {
        return Err(Error::InvalidArgument);
    }
}
*/