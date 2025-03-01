use anyhow::Result;
use chrono::{DateTime, Datelike, Local, NaiveDate, TimeZone, Utc};

use crate::config::ExchangeConfig;
use crate::email::{Email, EmailClient};

pub struct ExchangeClient {
    config: ExchangeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_calculate_quarter_date_range() {
        // Test Q1 (January-March)
        let q1_date = Local.ymd(2023, 2, 15).and_hms(12, 0, 0);
        let (start, end) = ExchangeClient::calculate_quarter_date_range(q1_date);
        
        assert_eq!(start, Utc.ymd(2023, 1, 1).and_hms(0, 0, 0));
        assert_eq!(end, Utc.ymd(2023, 3, 31).and_hms(23, 59, 59));
        
        // Test Q2 (April-June)
        let q2_date = Local.ymd(2023, 5, 15).and_hms(12, 0, 0);
        let (start, end) = ExchangeClient::calculate_quarter_date_range(q2_date);
        
        assert_eq!(start, Utc.ymd(2023, 4, 1).and_hms(0, 0, 0));
        assert_eq!(end, Utc.ymd(2023, 6, 30).and_hms(23, 59, 59));
        
        // Test Q3 (July-September)
        let q3_date = Local.ymd(2023, 8, 15).and_hms(12, 0, 0);
        let (start, end) = ExchangeClient::calculate_quarter_date_range(q3_date);
        
        assert_eq!(start, Utc.ymd(2023, 7, 1).and_hms(0, 0, 0));
        assert_eq!(end, Utc.ymd(2023, 9, 30).and_hms(23, 59, 59));
        
        // Test Q4 (October-December)
        let q4_date = Local.ymd(2023, 11, 15).and_hms(12, 0, 0);
        let (start, end) = ExchangeClient::calculate_quarter_date_range(q4_date);
        
        assert_eq!(start, Utc.ymd(2023, 10, 1).and_hms(0, 0, 0));
        assert_eq!(end, Utc.ymd(2023, 12, 31).and_hms(23, 59, 59));
        
        // Test leap year February (2024)
        let leap_year_date = Local.ymd(2024, 2, 15).and_hms(12, 0, 0);
        let (start, end) = ExchangeClient::calculate_quarter_date_range(leap_year_date);
        
        assert_eq!(start, Utc.ymd(2024, 1, 1).and_hms(0, 0, 0));
        assert_eq!(end, Utc.ymd(2024, 3, 31).and_hms(23, 59, 59));
    }
}

impl ExchangeClient {
    pub async fn new(config: &ExchangeConfig) -> Result<Self> {
        // In a real implementation, we would initialize the Exchange client here
        Ok(Self {
            config: config.clone(),
        })
    }
    
    fn get_quarter_date_range(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        Self::calculate_quarter_date_range(Local::now())
    }
    
    fn calculate_quarter_date_range(now: DateTime<Local>) -> (DateTime<Utc>, DateTime<Utc>) {
        let current_year = now.year();
        let current_quarter = (now.month() - 1) / 3 + 1;
        
        let quarter_start_month = (current_quarter - 1) * 3 + 1;
        let quarter_end_month = quarter_start_month + 2;
        
        let start_date = Utc.from_utc_datetime(
            &NaiveDate::from_ymd_opt(current_year, quarter_start_month, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
        
        // Get the last day of the month
        let end_month_last_day = match quarter_end_month {
            4 | 6 | 9 | 11 => 30,
            2 => {
                // February - check for leap year
                if current_year % 4 == 0 && (current_year % 100 != 0 || current_year % 400 == 0) {
                    29
                } else {
                    28
                }
            },
            _ => 31,
        };
        
        let end_date = Utc.from_utc_datetime(
            &NaiveDate::from_ymd_opt(current_year, quarter_end_month, end_month_last_day)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap()
        );
        
        (start_date, end_date)
    }
}

impl EmailClient for ExchangeClient {
    async fn fetch_current_quarter_emails(&self) -> Result<Vec<Email>> {
        let (_start_date, _end_date) = self.get_quarter_date_range();
        
        // TODO: Implement actual Exchange API call to fetch emails
        // For now, return mock data with realistic dates
        let now = Utc::now();
        let one_day = chrono::Duration::days(1);
        let two_days = chrono::Duration::days(2);
        let one_week = chrono::Duration::days(7);
        
        let emails = vec![
            Email {
                id: "1".to_string(),
                subject: "Project Update - Q2".to_string(),
                sender: "manager@company.com".to_string(),
                date: now - one_week,
                body: "Here's the latest update on our project progress...\n\nWe've completed the initial phase of development and are moving into testing. Please review the attached documents and provide feedback by the end of the week.\n\nThanks,\nProject Manager".to_string(),
            },
            Email {
                id: "2".to_string(),
                subject: "Team Meeting - Tomorrow".to_string(),
                sender: "team-lead@company.com".to_string(),
                date: now - one_day,
                body: "Reminder: We have a team meeting scheduled for tomorrow at 10 AM.\n\nAgenda:\n1. Project status updates\n2. Upcoming deadlines\n3. Resource allocation\n4. Open discussion\n\nPlease come prepared with your updates.\n\nRegards,\nTeam Lead".to_string(),
            },
            Email {
                id: "3".to_string(),
                subject: "Vacation Request".to_string(),
                sender: "hr@company.com".to_string(),
                date: now - two_days,
                body: "Your vacation request has been approved.\n\nDates: June 15-22, 2023\nTotal days: 5 business days\nRemaining PTO: 15 days\n\nPlease ensure all your tasks are properly handed over before your departure.\n\nBest regards,\nHR Department".to_string(),
            },
            Email {
                id: "4".to_string(),
                subject: "System Maintenance Notice".to_string(),
                sender: "it-support@company.com".to_string(),
                date: now,
                body: "Dear Team,\n\nPlease be informed that we will be performing system maintenance this weekend. The following systems will be unavailable from Saturday 8 PM to Sunday 2 AM:\n\n- Email servers\n- Internal documentation\n- Project management tools\n\nPlease plan your work accordingly.\n\nIT Support Team".to_string(),
            },
        ];
        
        Ok(emails)
    }
}
