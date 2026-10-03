use std::collections::HashMap;
use chrono::{Duration, Local, Utc};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn generate_uuid_v4<R: rand::Rng>(rng: &mut R) -> String {
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    bytes[6] = (bytes[6] & 0x0f) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // variant 1
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMockRule {
    pub column_name: String,
    pub data_type: String,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub nullable: bool,
    pub include: bool,
    pub generator_type: String,
    pub null_percentage: u32,
    pub custom_options: Option<String>,
    pub fk_target_table: Option<String>,
    pub fk_target_column: Option<String>,
    #[serde(default)]
    pub sample_fk_values: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableMockInspection {
    pub table_name: String,
    pub columns: Vec<ColumnMockRule>,
}

pub fn infer_generator_type(
    col_name: &str,
    data_type: &str,
    is_pk: bool,
    is_fk: bool,
) -> &'static str {
    let lower_name = col_name.to_lowercase();
    let lower_type = data_type.to_lowercase();

    if is_fk {
        return "fk_reference";
    }

    if is_pk {
        if lower_type.contains("serial")
            || lower_type.contains("auto_increment")
            || lower_type.contains("identity")
        {
            return "auto_increment";
        }
        if lower_type.contains("uuid") || lower_name.contains("uuid") || lower_name.contains("guid") {
            return "uuid";
        }
        if lower_type.contains("int") {
            return "auto_increment";
        }
    }

    // Name matching
    if lower_name.contains("email") || lower_name.contains("mail") {
        return "email";
    }
    if lower_name.contains("first_name") || lower_name.contains("fname") {
        return "first_name";
    }
    if lower_name.contains("last_name") || lower_name.contains("lname") || lower_name.contains("surname") {
        return "last_name";
    }
    if lower_name.contains("username") || lower_name.contains("user_name") || lower_name.contains("login") {
        return "username";
    }
    if lower_name.contains("full_name") || lower_name == "name" || lower_name.contains("author") || lower_name.contains("customer") {
        return "full_name";
    }

    // Contact & Address
    if lower_name.contains("phone") || lower_name.contains("mobile") || lower_name.contains("tel") || lower_name.contains("whatsapp") {
        return "phone";
    }
    if lower_name.contains("street") || lower_name.contains("address") || lower_name == "addr" {
        return "street_address";
    }
    if lower_name.contains("city") || lower_name.contains("kota") {
        return "city";
    }
    if lower_name.contains("country") || lower_name.contains("negara") {
        return "country";
    }
    if lower_name.contains("zip") || lower_name.contains("postal") {
        return "zip_code";
    }

    // Company & Job
    if lower_name.contains("company") || lower_name.contains("organization") || lower_name.contains("corp") {
        return "company";
    }
    if lower_name.contains("job") || lower_name.contains("title") || lower_name.contains("position") || lower_name.contains("role") {
        return "job_title";
    }

    // Internet & Media
    if lower_name.contains("avatar") || lower_name.contains("photo") || lower_name.contains("picture") || lower_name.contains("image") {
        return "avatar_url";
    }
    if lower_name.contains("url") || lower_name.contains("website") || lower_name.contains("link") {
        return "url";
    }
    if lower_name.contains("ip") || lower_name.contains("ipv4") {
        return "ip_v4";
    }

    // Finance & Numbers
    if lower_name.contains("price")
        || lower_name.contains("cost")
        || lower_name.contains("amount")
        || lower_name.contains("total")
        || lower_name.contains("salary")
        || lower_name.contains("balance")
        || lower_name.contains("revenue")
        || lower_name.contains("tax")
        || lower_name.contains("fee")
    {
        return "price";
    }
    if lower_name.contains("uuid") || lower_name.contains("guid") || lower_name.contains("token") || lower_name.contains("hash") {
        return "uuid";
    }
    if lower_name.contains("status") || lower_name.contains("state") || lower_name.contains("condition") {
        return "status";
    }

    // Booleans
    if lower_name.starts_with("is_")
        || lower_name.starts_with("has_")
        || lower_name == "active"
        || lower_name == "enabled"
        || lower_name == "deleted"
        || lower_type.contains("bool")
        || lower_type.contains("bit")
    {
        return "boolean";
    }

    // Timestamps & Dates
    if lower_name.contains("created")
        || lower_name.contains("registered")
        || lower_name.contains("joined")
        || lower_name.contains("updated")
        || lower_name.contains("modified")
    {
        return "timestamp_past";
    }
    if lower_name.contains("expiry") || lower_name.contains("due") || lower_name.contains("expires") {
        return "timestamp_future";
    }
    if lower_name.contains("birth") || lower_name.contains("dob") {
        return "date_past";
    }
    if lower_name.contains("date") || lower_name.contains("time") || lower_type.contains("date") || lower_type.contains("time") {
        return "timestamp_past";
    }

    // Descriptions & Text
    if lower_name.contains("desc")
        || lower_name.contains("bio")
        || lower_name.contains("note")
        || lower_name.contains("comment")
        || lower_name.contains("message")
        || lower_name.contains("body")
        || lower_name.contains("content")
        || lower_name.contains("summary")
    {
        return "lorem_sentence";
    }

    // Quantity / Integer numbers
    if lower_name.contains("count")
        || lower_name.contains("qty")
        || lower_name.contains("quantity")
        || lower_name.contains("age")
        || lower_name.contains("views")
        || lower_name.contains("likes")
        || lower_name.contains("score")
        || lower_name.contains("rating")
        || lower_name.contains("rank")
        || lower_name.contains("order")
    {
        return "integer";
    }

    // Data type based fallbacks
    if lower_type.contains("uuid") {
        return "uuid";
    }
    if lower_type.contains("int") || lower_type.contains("number") {
        return "integer";
    }
    if lower_type.contains("float") || lower_type.contains("decimal") || lower_type.contains("numeric") || lower_type.contains("real") || lower_type.contains("money") {
        return "price";
    }
    if lower_type.contains("json") {
        return "json_object";
    }

    "lorem_sentence"
}

// Mock Data Value Generator
pub struct MockDataGenerator {
    rng: StdRng,
}

impl MockDataGenerator {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    pub fn generate_row(&mut self, rules: &[ColumnMockRule], row_index: usize) -> HashMap<String, Value> {
        let mut row = HashMap::new();

        for rule in rules {
            if !rule.include {
                continue;
            }

            // Check nullable probability
            if rule.nullable && rule.null_percentage > 0 {
                let p: u32 = self.rng.gen_range(1..=100);
                if p <= rule.null_percentage {
                    row.insert(rule.column_name.clone(), Value::Null);
                    continue;
                }
            }

            let value = self.generate_value(rule, row_index);
            row.insert(rule.column_name.clone(), value);
        }

        row
    }

    pub fn generate_value(&mut self, rule: &ColumnMockRule, row_index: usize) -> Value {
        match rule.generator_type.as_str() {
            "auto_increment" => {
                Value::Number((row_index + 1).into())
            }
            "full_name" => {
                let first = self.pick_random(FIRST_NAMES);
                let last = self.pick_random(LAST_NAMES);
                Value::String(format!("{} {}", first, last))
            }
            "first_name" => {
                Value::String(self.pick_random(FIRST_NAMES).to_string())
            }
            "last_name" => {
                Value::String(self.pick_random(LAST_NAMES).to_string())
            }
            "email" => {
                let first = self.pick_random(FIRST_NAMES).to_lowercase();
                let last = self.pick_random(LAST_NAMES).to_lowercase();
                let domain = self.pick_random(DOMAINS);
                let num: u16 = self.rng.gen_range(10..999);
                Value::String(format!("{}.{}{}@{}", first, last, num, domain))
            }
            "username" => {
                let adj = self.pick_random(ADJECTIVES).to_lowercase();
                let noun = self.pick_random(NOUNS).to_lowercase();
                let num: u16 = self.rng.gen_range(1..99);
                Value::String(format!("{}{}{}", adj, noun, num))
            }
            "phone" => {
                let p1 = self.rng.gen_range(811..899);
                let p2 = self.rng.gen_range(1000..9999);
                let p3 = self.rng.gen_range(1000..9999);
                Value::String(format!("+62-{}-{}-{}", p1, p2, p3))
            }
            "street_address" => {
                let num = self.rng.gen_range(1..999);
                let street = self.pick_random(STREETS);
                Value::String(format!("Jl. {} No. {}", street, num))
            }
            "city" => {
                Value::String(self.pick_random(CITIES).to_string())
            }
            "country" => {
                Value::String(self.pick_random(COUNTRIES).to_string())
            }
            "zip_code" => {
                let zip = self.rng.gen_range(10000..99999);
                Value::String(zip.to_string())
            }
            "company" => {
                let name = self.pick_random(COMPANIES);
                let suffix = self.pick_random(COMPANY_SUFFIXES);
                Value::String(format!("{} {}", name, suffix))
            }
            "job_title" => {
                Value::String(self.pick_random(JOB_TITLES).to_string())
            }
            "price" => {
                let dollars = self.rng.gen_range(5..999);
                let cents = self.rng.gen_range(0..99);
                let val = (dollars as f64) + (cents as f64 / 100.0);
                serde_json::Number::from_f64(val)
                    .map(Value::Number)
                    .unwrap_or(Value::String(format!("{:.2}", val)))
            }
            "integer" => {
                let min = 1;
                let max = 1000;
                let val = self.rng.gen_range(min..=max);
                Value::Number(val.into())
            }
            "boolean" => {
                Value::Bool(self.rng.gen_bool(0.75))
            }
            "uuid" => {
                Value::String(generate_uuid_v4(&mut self.rng))
            }
            "timestamp_past" => {
                let days_ago = self.rng.gen_range(1..730);
                let dt = Utc::now() - Duration::days(days_ago);
                Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            "timestamp_future" => {
                let days_ahead = self.rng.gen_range(1..365);
                let dt = Utc::now() + Duration::days(days_ahead);
                Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string())
            }
            "date_past" => {
                let days_ago = self.rng.gen_range(365 * 18..365 * 60);
                let dt = Local::now().naive_local().date() - Duration::days(days_ago);
                Value::String(dt.format("%Y-%m-%d").to_string())
            }
            "url" => {
                let proto = "https";
                let domain = self.pick_random(DOMAINS);
                let slug = self.pick_random(NOUNS).to_lowercase();
                Value::String(format!("{}://{}/{}", proto, domain, slug))
            }
            "avatar_url" => {
                let id = self.rng.gen_range(1..100);
                Value::String(format!("https://i.pravatar.cc/150?img={}", id))
            }
            "ip_v4" => {
                let a = self.rng.gen_range(10..220);
                let b = self.rng.gen_range(1..254);
                let c = self.rng.gen_range(1..254);
                let d = self.rng.gen_range(1..254);
                Value::String(format!("{}.{}.{}.{}", a, b, c, d))
            }
            "status" => {
                Value::String(self.pick_random(STATUSES).to_string())
            }
            "lorem_sentence" => {
                Value::String(self.pick_random(LOREM_SENTENCES).to_string())
            }
            "json_object" => {
                let theme = self.pick_random(&["dark", "light", "system"]);
                let notifications = self.rng.gen_bool(0.8);
                let role = self.pick_random(&["admin", "editor", "viewer"]);
                serde_json::json!({
                    "theme": theme,
                    "notifications": notifications,
                    "role": role,
                    "v": 1
                })
            }
            "custom_list" => {
                if let Some(opts) = &rule.custom_options {
                    let items: Vec<&str> = opts.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
                    if !items.is_empty() {
                        return Value::String(self.pick_random(&items).to_string());
                    }
                }
                Value::String("Default".into())
            }
            "fk_reference" => {
                if !rule.sample_fk_values.is_empty() {
                    let idx = self.rng.gen_range(0..rule.sample_fk_values.len());
                    return rule.sample_fk_values[idx].clone();
                }
                // Fallback if no FK values available
                if rule.data_type.to_lowercase().contains("int") {
                    Value::Number(1.into())
                } else if rule.data_type.to_lowercase().contains("uuid") {
                    Value::String(generate_uuid_v4(&mut self.rng))
                } else {
                    Value::String("1".into())
                }
            }
            _ => {
                Value::String(self.pick_random(LOREM_SENTENCES).to_string())
            }
        }
    }

    fn pick_random<'a>(&mut self, list: &[&'a str]) -> &'a str {
        if list.is_empty() {
            return "";
        }
        let idx = self.rng.gen_range(0..list.len());
        list[idx]
    }
}

// Seed datasets
const FIRST_NAMES: &[&str] = &[
    "Aditya", "Budi", "Siti", "Dewi", "Rian", "Agus", "Putri", "Reza", "Fajar", "Indah",
    "Alexander", "Emma", "Liam", "Olivia", "Noah", "Sophia", "Lucas", "Mia", "Ethan", "Isabella",
    "Mason", "Amelia", "James", "Harper", "Benjamin", "Evelyn", "Elijah", "Abigail", "Daniel", "Emily"
];

const LAST_NAMES: &[&str] = &[
    "Pratama", "Santoso", "Wijaya", "Kusuma", "Hidayat", "Saputra", "Lestari", "Nugroho", "Wulandari", "Setiawan",
    "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez",
    "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson", "Thomas", "Taylor", "Moore", "Jackson", "Martin"
];

const DOMAINS: &[&str] = &[
    "fugdb.dev", "gmail.com", "yahoo.com", "outlook.com", "icloud.com", "company.io", "techlabs.org", "enterprise.net"
];

const ADJECTIVES: &[&str] = &[
    "Swift", "Brave", "Clever", "Silent", "Neon", "Cyber", "Mighty", "Hyper", "Stellar", "Atomic",
    "Quantum", "Dynamic", "Golden", "Cosmic", "Vibrant", "Shadow", "Alpha", "Prime", "Ultra", "Apex"
];

const NOUNS: &[&str] = &[
    "Coder", "Hacker", "Falcon", "Tiger", "Dragon", "Phoenix", "Panda", "Wizard", "Ninja", "Knight",
    "Runner", "Pilot", "Explorer", "Spark", "Nova", "Matrix", "Galaxy", "Rocket", "Viper", "Titan"
];

const STREETS: &[&str] = &[
    "Sudirman", "Thamrin", "Gatot Subroto", "Rasuna Said", "Kuningan", "Diponegoro", "Asia Afrika", "Malioboro",
    "Market Street", "Broadway", "Sunset Blvd", "Wall Street", "Oxford Street", "Fifth Avenue", "King Street"
];

const CITIES: &[&str] = &[
    "Jakarta", "Surabaya", "Bandung", "Medan", "Semarang", "Yogyakarta", "Bali", "Makassar",
    "Singapore", "Tokyo", "London", "New York", "San Francisco", "Sydney", "Berlin", "Toronto", "Paris"
];

const COUNTRIES: &[&str] = &[
    "Indonesia", "Singapore", "Malaysia", "Japan", "United States", "United Kingdom", "Germany", "Australia", "Canada", "Netherlands"
];

const COMPANIES: &[&str] = &[
    "TechNova", "Apex Global", "Nusantara Digital", "Solusi Prima", "Inovasi Maju", "CyberCore", "DataSphere", "CloudSync", "Quantum Leap", "Vanguard Labs"
];

const COMPANY_SUFFIXES: &[&str] = &[
    "Inc", "Corp", "Ltd", "LLC", "Technologies", "Solutions", "Group", "International", "PT", "Ventures"
];

const JOB_TITLES: &[&str] = &[
    "Senior Software Engineer", "Full Stack Developer", "DevOps Specialist", "Product Manager",
    "Data Scientist", "UI/UX Designer", "QA Automation Engineer", "Database Administrator",
    "Cybersecurity Analyst", "Solutions Architect", "Technical Lead", "Chief Technology Officer"
];

const STATUSES: &[&str] = &[
    "active", "pending", "completed", "archived", "inactive", "in_progress", "draft", "verified"
];

const LOREM_SENTENCES: &[&str] = &[
    "High performance database queries executed seamlessly with zero overhead.",
    "Streamlined database workflow designed for modern engineering teams.",
    "Comprehensive schema documentation generated with one click.",
    "Ultra-low latency data transfer directly across disparate database engines.",
    "Empowering developers with real-time interactive schema exploration.",
    "Automated data verification and foreign key relational constraint validation.",
    "Robust enterprise grade database client built with Tauri v2 and Rust."
];
