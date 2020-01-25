#[derive(Debug)]
#[derive(PartialEq)]
struct FatDate {
    day: u8,
    month: u8,
    year: u16,
}

impl FatDate {
    pub fn deserialize(data: u16) -> FatDate {
        FatDate {
            day: (data & 0b11111) as u8,
            month: ((data >> 5) & 0b1111) as u8,
            year: ((data) >> 9) + 1980,
        }
    }

    pub fn set_day(&mut self, day: u8) {
        if (0 < day) && (day <= 31) {
            self.day = day;
        } 
    }

    pub fn set_month(&mut self, month: u8) {
        if (0 < month) && (month <= 12) {
            self.month = month;
        }
    }

    pub fn set_year(&mut self, year: u16) {
        if (1980 <= year) && (year <= 2107) {
            self.year = year - 1980;
        }
    }

    pub fn day(&self) -> u8 {
        self.day
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn year(&self) -> u16 {
        self.year
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
struct FatTime {
    second: u8,
    minute: u8,
    hour: u8,
}

impl FatTime {
    pub fn deserialize(data: u16) -> FatTime {
        FatTime {
            second: ((data & 0b11111) * 2) as u8,
            minute: ((data >> 5) & 0b111111) as u8,
            hour: (data) >> 11) as u8,
        }
    }

    pub fn set_second(&mut self, second: u8) {
        if (0 <= second) && (second <= 59) {
            self.second = second;
        } 
    }

    pub fn set_minute(&mut self, minute: u8) {
        if (0 <= minute) && (minute <= 59) {
            self.minute = minute;
        }
    }

    pub fn set_hour(&mut self, hour: u8) {
        if (0 <= hour) && (hour <= 23) {
            self.hour = hour;
        }
    }

    pub fn second(&self) -> u8 {
        self.second
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn hour(&self) -> u16 {
        self.hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fat_date_test() {
        let date = 15 | (4 << 5) | (17 << 9);
        assert_eq!(FatDate::deserialize(date), FatDate { day: 15, month: 4, year: 1997 });
    }
}