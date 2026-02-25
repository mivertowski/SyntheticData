//! Multi-cultural name generation for realistic user personas.
//!
//! Provides name pools for various cultures to generate realistic
//! user names, IDs, and email addresses.

use rand::seq::IndexedRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cultural background for name generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NameCulture {
    /// Western US names (default)
    #[default]
    WesternUs,
    /// German names
    German,
    /// French names
    French,
    /// Chinese names
    Chinese,
    /// Japanese names
    Japanese,
    /// Indian names
    Indian,
    /// Hispanic/Latino names
    Hispanic,
}

impl NameCulture {
    /// Get all available cultures.
    pub fn all() -> &'static [NameCulture] {
        &[
            NameCulture::WesternUs,
            NameCulture::German,
            NameCulture::French,
            NameCulture::Chinese,
            NameCulture::Japanese,
            NameCulture::Indian,
            NameCulture::Hispanic,
        ]
    }
}

/// A generated person name with components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonName {
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
    /// Full display name
    pub display_name: String,
    /// Culture of origin
    pub culture: NameCulture,
    /// Whether the name is typically male
    pub is_male: bool,
}

impl PersonName {
    /// Generate a user ID from the name (e.g., "JSMITH001").
    pub fn to_user_id(&self, index: usize) -> String {
        let first_initial = self.first_name.chars().next().unwrap_or('X');
        let last_part: String = self
            .last_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .take(5)
            .collect();
        format!("{}{}{:03}", first_initial, last_part.to_uppercase(), index)
    }

    /// Generate an email address from the name.
    pub fn to_email(&self, domain: &str) -> String {
        let first = self
            .first_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();
        let last = self
            .last_name
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase();
        format!("{}.{}@{}", first, last, domain)
    }
}

/// Pool of names for a specific culture.
#[derive(Debug, Clone)]
pub struct NamePool {
    /// Male first names
    pub first_names_male: Vec<String>,
    /// Female first names
    pub first_names_female: Vec<String>,
    /// Last names / family names
    pub last_names: Vec<String>,
    /// The culture this pool represents
    pub culture: NameCulture,
}

/// Helper to convert `&[&str]` to `Vec<String>` concisely in factory methods.
fn sv(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

impl NamePool {
    /// Create a Western US name pool.
    pub fn western_us() -> Self {
        Self {
            culture: NameCulture::WesternUs,
            first_names_male: sv(&[
                "James",
                "John",
                "Robert",
                "Michael",
                "William",
                "David",
                "Richard",
                "Joseph",
                "Thomas",
                "Christopher",
                "Charles",
                "Daniel",
                "Matthew",
                "Anthony",
                "Mark",
                "Donald",
                "Steven",
                "Paul",
                "Andrew",
                "Joshua",
                "Kenneth",
                "Kevin",
                "Brian",
                "George",
                "Timothy",
                "Ronald",
                "Edward",
                "Jason",
                "Jeffrey",
                "Ryan",
                "Jacob",
                "Gary",
                "Nicholas",
                "Eric",
                "Jonathan",
                "Stephen",
                "Larry",
                "Justin",
                "Scott",
                "Brandon",
                "Benjamin",
                "Samuel",
                "Raymond",
                "Gregory",
                "Frank",
                "Alexander",
                "Patrick",
                "Jack",
                "Dennis",
                "Jerry",
                "Tyler",
                "Aaron",
                "Jose",
                "Adam",
                "Nathan",
            ]),
            first_names_female: sv(&[
                "Mary",
                "Patricia",
                "Jennifer",
                "Linda",
                "Barbara",
                "Elizabeth",
                "Susan",
                "Jessica",
                "Sarah",
                "Karen",
                "Lisa",
                "Nancy",
                "Betty",
                "Margaret",
                "Sandra",
                "Ashley",
                "Kimberly",
                "Emily",
                "Donna",
                "Michelle",
                "Dorothy",
                "Carol",
                "Amanda",
                "Melissa",
                "Deborah",
                "Stephanie",
                "Rebecca",
                "Sharon",
                "Laura",
                "Cynthia",
                "Kathleen",
                "Amy",
                "Angela",
                "Shirley",
                "Anna",
                "Brenda",
                "Pamela",
                "Emma",
                "Nicole",
                "Helen",
                "Samantha",
                "Katherine",
                "Christine",
                "Debra",
                "Rachel",
                "Carolyn",
                "Janet",
                "Catherine",
                "Maria",
                "Heather",
                "Diane",
                "Ruth",
                "Julie",
            ]),
            last_names: sv(&[
                "Smith",
                "Johnson",
                "Williams",
                "Brown",
                "Jones",
                "Garcia",
                "Miller",
                "Davis",
                "Rodriguez",
                "Martinez",
                "Hernandez",
                "Lopez",
                "Gonzalez",
                "Wilson",
                "Anderson",
                "Thomas",
                "Taylor",
                "Moore",
                "Jackson",
                "Martin",
                "Lee",
                "Perez",
                "Thompson",
                "White",
                "Harris",
                "Sanchez",
                "Clark",
                "Ramirez",
                "Lewis",
                "Robinson",
                "Walker",
                "Young",
                "Allen",
                "King",
                "Wright",
                "Scott",
                "Torres",
                "Nguyen",
                "Hill",
                "Flores",
                "Green",
                "Adams",
                "Nelson",
                "Baker",
                "Hall",
                "Rivera",
                "Campbell",
                "Mitchell",
                "Carter",
                "Roberts",
                "Turner",
                "Phillips",
                "Evans",
                "Parker",
                "Edwards",
                "Collins",
            ]),
        }
    }

    /// Create a German name pool.
    pub fn german() -> Self {
        Self {
            culture: NameCulture::German,
            first_names_male: sv(&[
                "Hans",
                "Klaus",
                "Wolfgang",
                "Dieter",
                "Jürgen",
                "Peter",
                "Michael",
                "Thomas",
                "Andreas",
                "Stefan",
                "Markus",
                "Christian",
                "Martin",
                "Frank",
                "Bernd",
                "Uwe",
                "Ralf",
                "Werner",
                "Heinz",
                "Helmut",
                "Gerhard",
                "Manfred",
                "Horst",
                "Karl",
                "Heinrich",
                "Friedrich",
                "Wilhelm",
                "Otto",
                "Matthias",
                "Tobias",
                "Sebastian",
                "Florian",
                "Alexander",
                "Maximilian",
                "Felix",
                "Lukas",
                "Jonas",
                "Leon",
                "Paul",
                "Philipp",
                "Tim",
                "Jan",
                "Nico",
                "Erik",
                "Lars",
                "Sven",
                "Kai",
                "Olaf",
                "Rainer",
            ]),
            first_names_female: sv(&[
                "Anna",
                "Maria",
                "Elisabeth",
                "Ursula",
                "Helga",
                "Monika",
                "Petra",
                "Sabine",
                "Karin",
                "Renate",
                "Ingrid",
                "Brigitte",
                "Gisela",
                "Erika",
                "Christa",
                "Andrea",
                "Claudia",
                "Susanne",
                "Birgit",
                "Heike",
                "Martina",
                "Nicole",
                "Stefanie",
                "Julia",
                "Katharina",
                "Christina",
                "Sandra",
                "Melanie",
                "Daniela",
                "Anja",
                "Tanja",
                "Simone",
                "Silke",
                "Nadine",
                "Yvonne",
                "Manuela",
                "Sonja",
                "Michaela",
                "Angelika",
                "Barbara",
                "Gabriele",
                "Beate",
                "Doris",
                "Eva",
                "Franziska",
                "Lena",
                "Hannah",
                "Sophie",
                "Leonie",
                "Laura",
            ]),
            last_names: sv(&[
                "Müller",
                "Schmidt",
                "Schneider",
                "Fischer",
                "Weber",
                "Meyer",
                "Wagner",
                "Becker",
                "Schulz",
                "Hoffmann",
                "Schäfer",
                "Koch",
                "Bauer",
                "Richter",
                "Klein",
                "Wolf",
                "Schröder",
                "Neumann",
                "Schwarz",
                "Zimmermann",
                "Braun",
                "Krüger",
                "Hofmann",
                "Hartmann",
                "Lange",
                "Schmitt",
                "Werner",
                "Schmitz",
                "Krause",
                "Meier",
                "Lehmann",
                "Schmid",
                "Schulze",
                "Maier",
                "Köhler",
                "Herrmann",
                "König",
                "Walter",
                "Mayer",
                "Huber",
                "Kaiser",
                "Fuchs",
                "Peters",
                "Lang",
                "Scholz",
                "Möller",
                "Weiß",
                "Jung",
                "Hahn",
                "Schubert",
            ]),
        }
    }

    /// Create a French name pool.
    pub fn french() -> Self {
        Self {
            culture: NameCulture::French,
            first_names_male: sv(&[
                "Jean",
                "Pierre",
                "Michel",
                "André",
                "Philippe",
                "Jacques",
                "Bernard",
                "Alain",
                "François",
                "Robert",
                "Marcel",
                "René",
                "Louis",
                "Claude",
                "Daniel",
                "Yves",
                "Christian",
                "Patrick",
                "Nicolas",
                "Julien",
                "Thomas",
                "Antoine",
                "Alexandre",
                "Maxime",
                "Lucas",
                "Hugo",
                "Théo",
                "Mathieu",
                "Guillaume",
                "Laurent",
                "Olivier",
                "Christophe",
                "Sébastien",
                "Frédéric",
                "Vincent",
                "David",
                "Eric",
                "Pascal",
                "Gilles",
                "Thierry",
                "Stéphane",
                "Bruno",
                "Dominique",
                "Serge",
                "Maurice",
                "Henri",
                "Paul",
                "Charles",
                "Emmanuel",
                "Raphaël",
            ]),
            first_names_female: sv(&[
                "Marie",
                "Jeanne",
                "Françoise",
                "Monique",
                "Catherine",
                "Nathalie",
                "Isabelle",
                "Sylvie",
                "Anne",
                "Martine",
                "Nicole",
                "Christine",
                "Sophie",
                "Valérie",
                "Julie",
                "Camille",
                "Léa",
                "Manon",
                "Emma",
                "Chloé",
                "Inès",
                "Sarah",
                "Laura",
                "Louise",
                "Jade",
                "Alice",
                "Lola",
                "Margot",
                "Charlotte",
                "Clara",
                "Pauline",
                "Marine",
                "Aurélie",
                "Céline",
                "Sandrine",
                "Virginie",
                "Stéphanie",
                "Élodie",
                "Delphine",
                "Laurence",
                "Brigitte",
                "Jacqueline",
                "Simone",
                "Denise",
                "Madeleine",
                "Thérèse",
                "Hélène",
                "Élise",
                "Juliette",
                "Marguerite",
            ]),
            last_names: sv(&[
                "Martin",
                "Bernard",
                "Dubois",
                "Thomas",
                "Robert",
                "Richard",
                "Petit",
                "Durand",
                "Leroy",
                "Moreau",
                "Simon",
                "Laurent",
                "Lefebvre",
                "Michel",
                "Garcia",
                "David",
                "Bertrand",
                "Roux",
                "Vincent",
                "Fournier",
                "Morel",
                "Girard",
                "André",
                "Lefèvre",
                "Mercier",
                "Dupont",
                "Lambert",
                "Bonnet",
                "François",
                "Martinez",
                "Legrand",
                "Garnier",
                "Faure",
                "Rousseau",
                "Blanc",
                "Guérin",
                "Muller",
                "Henry",
                "Roussel",
                "Nicolas",
                "Perrin",
                "Morin",
                "Mathieu",
                "Clément",
                "Gauthier",
                "Dumont",
                "Lopez",
                "Fontaine",
                "Chevalier",
                "Robin",
            ]),
        }
    }

    /// Create a Chinese name pool.
    pub fn chinese() -> Self {
        Self {
            culture: NameCulture::Chinese,
            first_names_male: sv(&[
                "Wei", "Fang", "Lei", "Jun", "Jian", "Hao", "Chen", "Yang", "Ming", "Tao", "Long",
                "Feng", "Bin", "Qiang", "Gang", "Hui", "Peng", "Xiang", "Bo", "Chao", "Dong",
                "Liang", "Ning", "Kai", "Jie", "Yong", "Hai", "Lin", "Wen", "Zheng", "Hong", "Xin",
                "Da", "Zhi", "Guang", "Cheng", "Yi", "Sheng", "Biao", "Ping", "Yun", "Song",
                "Chang", "Kang", "Rui", "Nan", "Jia", "Xiao", "Yu", "Hua",
            ]),
            first_names_female: sv(&[
                "Fang", "Min", "Jing", "Li", "Yan", "Hong", "Juan", "Mei", "Ying", "Xia", "Hui",
                "Lin", "Ling", "Ping", "Dan", "Yun", "Na", "Qian", "Xin", "Ya", "Wei", "Wen",
                "Jie", "Qing", "Yu", "Hua", "Yue", "Xue", "Lan", "Zhen", "Rong", "Shu", "Fei",
                "Lei", "Shan", "Ting", "Ni", "Ying", "Chen", "Huan", "Lu", "Ai", "Xiao", "Xiang",
                "Yao", "Meng", "Qi", "Jun", "Bei", "Zhi",
            ]),
            last_names: sv(&[
                "Wang", "Li", "Zhang", "Liu", "Chen", "Yang", "Huang", "Zhao", "Wu", "Zhou", "Xu",
                "Sun", "Ma", "Zhu", "Hu", "Guo", "He", "Lin", "Gao", "Luo", "Zheng", "Liang",
                "Xie", "Tang", "Song", "Xu", "Han", "Deng", "Feng", "Cao", "Peng", "Xiao", "Cheng",
                "Yuan", "Tian", "Dong", "Pan", "Cai", "Jiang", "Wei", "Yu", "Du", "Ye", "Shi",
                "Lu", "Shen", "Su", "Jia", "Fan", "Jin",
            ]),
        }
    }

    /// Create a Japanese name pool.
    pub fn japanese() -> Self {
        Self {
            culture: NameCulture::Japanese,
            first_names_male: sv(&[
                "Hiroshi", "Takeshi", "Kenji", "Yuki", "Kazuki", "Ryota", "Daiki", "Shota", "Yuto",
                "Kenta", "Haruto", "Sota", "Riku", "Yuma", "Kaito", "Ren", "Hayato", "Takumi",
                "Kouki", "Ryuu", "Naoki", "Tsubasa", "Yuuki", "Akira", "Satoshi", "Makoto",
                "Tetsuya", "Masaki", "Shin", "Kei", "Daisuke", "Shunsuke", "Tomoya", "Yusuke",
                "Tatsuya", "Katsuki", "Shun", "Yamato", "Koji", "Hideo", "Takahiro", "Noboru",
                "Shinji", "Osamu", "Minoru", "Hideki", "Jun", "Masaru", "Ken", "Ryo",
            ]),
            first_names_female: sv(&[
                "Yuki", "Sakura", "Hana", "Yui", "Mio", "Rin", "Aoi", "Mei", "Saki", "Miku",
                "Nanami", "Ayaka", "Misaki", "Haruka", "Momoka", "Rina", "Yuna", "Hinata",
                "Koharu", "Miyu", "Akari", "Hikari", "Kaede", "Natsuki", "Mai", "Ami", "Aya",
                "Emi", "Kana", "Megumi", "Tomoko", "Yoko", "Keiko", "Naomi", "Mayumi", "Chika",
                "Nana", "Risa", "Asuka", "Fumiko", "Kyoko", "Reiko", "Noriko", "Sachiko", "Mariko",
                "Shiori", "Midori", "Kanako", "Minami", "Eriko",
            ]),
            last_names: sv(&[
                "Sato",
                "Suzuki",
                "Takahashi",
                "Tanaka",
                "Watanabe",
                "Ito",
                "Yamamoto",
                "Nakamura",
                "Kobayashi",
                "Kato",
                "Yoshida",
                "Yamada",
                "Sasaki",
                "Yamaguchi",
                "Matsumoto",
                "Inoue",
                "Kimura",
                "Hayashi",
                "Shimizu",
                "Yamazaki",
                "Mori",
                "Abe",
                "Ikeda",
                "Hashimoto",
                "Yamashita",
                "Ishikawa",
                "Nakajima",
                "Maeda",
                "Fujita",
                "Ogawa",
                "Goto",
                "Okada",
                "Hasegawa",
                "Murakami",
                "Kondo",
                "Ishii",
                "Saito",
                "Sakamoto",
                "Endo",
                "Aoki",
                "Fujii",
                "Nishimura",
                "Fukuda",
                "Ota",
                "Miura",
                "Okamoto",
                "Matsuda",
                "Nakagawa",
                "Fujiwara",
                "Kawamura",
            ]),
        }
    }

    /// Create an Indian name pool.
    pub fn indian() -> Self {
        Self {
            culture: NameCulture::Indian,
            first_names_male: sv(&[
                "Raj", "Amit", "Vikram", "Rahul", "Sanjay", "Arun", "Suresh", "Rajesh", "Deepak",
                "Vijay", "Prakash", "Manoj", "Sunil", "Anil", "Ravi", "Ashok", "Ramesh", "Mukesh",
                "Sandeep", "Ajay", "Naveen", "Pradeep", "Sachin", "Nitin", "Vinod", "Rakesh",
                "Srinivas", "Ganesh", "Krishna", "Mohan", "Kiran", "Venkat", "Hari", "Shankar",
                "Dinesh", "Mahesh", "Satish", "Girish", "Naresh", "Harish", "Pavan", "Arjun",
                "Anand", "Vivek", "Rohit", "Gaurav", "Kunal", "Vishal", "Akhil", "Dev",
            ]),
            first_names_female: sv(&[
                "Priya",
                "Anita",
                "Sunita",
                "Kavita",
                "Neha",
                "Pooja",
                "Anjali",
                "Deepa",
                "Meena",
                "Rekha",
                "Lakshmi",
                "Padma",
                "Gita",
                "Sita",
                "Rani",
                "Devi",
                "Shalini",
                "Nisha",
                "Swati",
                "Preeti",
                "Divya",
                "Shreya",
                "Aishwarya",
                "Ritu",
                "Seema",
                "Jyoti",
                "Shweta",
                "Pallavi",
                "Rashmi",
                "Smita",
                "Varsha",
                "Archana",
                "Asha",
                "Manisha",
                "Usha",
                "Vandana",
                "Geeta",
                "Rita",
                "Maya",
                "Radha",
                "Sapna",
                "Megha",
                "Nikita",
                "Tanvi",
                "Aditi",
                "Bhavna",
                "Chitra",
                "Komal",
                "Madhuri",
                "Parul",
            ]),
            last_names: sv(&[
                "Sharma",
                "Patel",
                "Singh",
                "Kumar",
                "Gupta",
                "Joshi",
                "Verma",
                "Shah",
                "Mehta",
                "Reddy",
                "Rao",
                "Iyer",
                "Nair",
                "Menon",
                "Pillai",
                "Das",
                "Bose",
                "Sen",
                "Chatterjee",
                "Banerjee",
                "Mukherjee",
                "Ghosh",
                "Roy",
                "Dutta",
                "Kapoor",
                "Malhotra",
                "Agarwal",
                "Sinha",
                "Thakur",
                "Saxena",
                "Mishra",
                "Pandey",
                "Trivedi",
                "Desai",
                "Modi",
                "Kulkarni",
                "Patil",
                "Kaur",
                "Chopra",
                "Khanna",
                "Bhatia",
                "Choudhury",
                "Rajan",
                "Subramaniam",
                "Venkatesh",
                "Naidu",
                "Hegde",
                "Shukla",
                "Prasad",
                "Murthy",
            ]),
        }
    }

    /// Create a Hispanic name pool.
    pub fn hispanic() -> Self {
        Self {
            culture: NameCulture::Hispanic,
            first_names_male: sv(&[
                "José",
                "Juan",
                "Carlos",
                "Luis",
                "Miguel",
                "Francisco",
                "Antonio",
                "Manuel",
                "Pedro",
                "Javier",
                "Fernando",
                "Rafael",
                "Alejandro",
                "Ricardo",
                "Eduardo",
                "Roberto",
                "Daniel",
                "Pablo",
                "Sergio",
                "Jorge",
                "Andrés",
                "Raúl",
                "Diego",
                "Enrique",
                "Óscar",
                "Adrián",
                "Víctor",
                "Martín",
                "Gabriel",
                "Álvaro",
                "Iván",
                "Mario",
                "César",
                "Héctor",
                "Alberto",
                "Gustavo",
                "Arturo",
                "Ramón",
                "Hugo",
                "Salvador",
                "Ernesto",
                "Guillermo",
                "Ignacio",
                "Jaime",
                "Felipe",
                "Tomás",
                "Santiago",
                "Mateo",
                "Sebastián",
                "Nicolás",
            ]),
            first_names_female: sv(&[
                "María",
                "Carmen",
                "Ana",
                "Laura",
                "Rosa",
                "Isabel",
                "Elena",
                "Patricia",
                "Teresa",
                "Lucia",
                "Pilar",
                "Dolores",
                "Paula",
                "Sara",
                "Marta",
                "Julia",
                "Cristina",
                "Claudia",
                "Andrea",
                "Mónica",
                "Gloria",
                "Beatriz",
                "Alicia",
                "Rocío",
                "Victoria",
                "Silvia",
                "Eva",
                "Raquel",
                "Adriana",
                "Lorena",
                "Gabriela",
                "Marina",
                "Sandra",
                "Verónica",
                "Natalia",
                "Carolina",
                "Diana",
                "Alejandra",
                "Cecilia",
                "Daniela",
                "Sofía",
                "Valentina",
                "Camila",
                "Isabella",
                "Mariana",
                "Fernanda",
                "Paola",
                "Liliana",
                "Angela",
                "Inés",
            ]),
            last_names: sv(&[
                "García",
                "Rodríguez",
                "Martínez",
                "López",
                "González",
                "Hernández",
                "Pérez",
                "Sánchez",
                "Ramírez",
                "Torres",
                "Flores",
                "Rivera",
                "Gómez",
                "Díaz",
                "Reyes",
                "Cruz",
                "Morales",
                "Ortiz",
                "Gutiérrez",
                "Chávez",
                "Ramos",
                "Vargas",
                "Castillo",
                "Jiménez",
                "Moreno",
                "Romero",
                "Herrera",
                "Medina",
                "Aguilar",
                "Vega",
                "Castro",
                "Mendoza",
                "Ruiz",
                "Fernández",
                "Álvarez",
                "Muñoz",
                "Rojas",
                "Silva",
                "Suárez",
                "Delgado",
                "Navarro",
                "Santos",
                "Molina",
                "Espinoza",
                "Guerrero",
                "Cabrera",
                "Campos",
                "Cortés",
                "Salazar",
                "Luna",
            ]),
        }
    }

    /// Get a name pool for a specific culture.
    pub fn for_culture(culture: NameCulture) -> Self {
        match culture {
            NameCulture::WesternUs => Self::western_us(),
            NameCulture::German => Self::german(),
            NameCulture::French => Self::french(),
            NameCulture::Chinese => Self::chinese(),
            NameCulture::Japanese => Self::japanese(),
            NameCulture::Indian => Self::indian(),
            NameCulture::Hispanic => Self::hispanic(),
        }
    }

    /// Generate a random name from this pool.
    pub fn generate_name(&self, rng: &mut impl Rng) -> PersonName {
        let is_male = rng.random_bool(0.5);

        let first_name = if is_male {
            self.first_names_male
                .choose(rng)
                .expect("non-empty name list")
        } else {
            self.first_names_female
                .choose(rng)
                .expect("non-empty name list")
        };

        let last_name = self.last_names.choose(rng).expect("non-empty name list");

        PersonName {
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            display_name: format!("{} {}", first_name, last_name),
            culture: self.culture,
            is_male,
        }
    }

    /// Create a `NamePool` from a `CultureConfig` loaded from a country pack.
    pub fn from_culture_config(
        config: &crate::country::schema::CultureConfig,
        culture: NameCulture,
    ) -> Self {
        Self {
            culture,
            first_names_male: config.male_first_names.clone(),
            first_names_female: config.female_first_names.clone(),
            last_names: config.last_names.clone(),
        }
    }
}

/// Multi-culture name generator with weighted distribution.
#[derive(Debug, Clone)]
pub struct MultiCultureNameGenerator {
    pools: HashMap<NameCulture, NamePool>,
    distribution: Vec<(NameCulture, f64)>,
    email_domain: String,
}

impl Default for MultiCultureNameGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiCultureNameGenerator {
    /// Create a new generator with default distribution.
    pub fn new() -> Self {
        let mut pools = HashMap::new();
        for culture in NameCulture::all() {
            pools.insert(*culture, NamePool::for_culture(*culture));
        }

        Self {
            pools,
            distribution: vec![
                (NameCulture::WesternUs, 0.40),
                (NameCulture::Hispanic, 0.20),
                (NameCulture::German, 0.10),
                (NameCulture::French, 0.05),
                (NameCulture::Chinese, 0.10),
                (NameCulture::Japanese, 0.05),
                (NameCulture::Indian, 0.10),
            ],
            email_domain: "company.com".to_string(),
        }
    }

    /// Create a generator with custom distribution.
    pub fn with_distribution(distribution: Vec<(NameCulture, f64)>) -> Self {
        let mut gen = Self::new();
        gen.distribution = distribution;
        gen
    }

    /// Set the email domain.
    pub fn with_email_domain(mut self, domain: &str) -> Self {
        self.email_domain = domain.to_string();
        self
    }

    /// Set the email domain (mutable reference).
    pub fn set_email_domain(&mut self, domain: &str) {
        self.email_domain = domain.to_string();
    }

    /// Get the email domain.
    pub fn email_domain(&self) -> &str {
        &self.email_domain
    }

    /// Select a culture based on the distribution.
    fn select_culture(&self, rng: &mut impl Rng) -> NameCulture {
        let roll: f64 = rng.random();
        let mut cumulative = 0.0;

        for (culture, weight) in &self.distribution {
            cumulative += weight;
            if roll < cumulative {
                return *culture;
            }
        }

        // Fallback to default
        NameCulture::WesternUs
    }

    /// Generate a random name from the weighted distribution.
    pub fn generate_name(&self, rng: &mut impl Rng) -> PersonName {
        let culture = self.select_culture(rng);
        self.pools
            .get(&culture)
            .map(|pool| pool.generate_name(rng))
            .unwrap_or_else(|| NamePool::western_us().generate_name(rng))
    }

    /// Generate a name with a specific culture.
    pub fn generate_name_for_culture(
        &self,
        culture: NameCulture,
        rng: &mut impl Rng,
    ) -> PersonName {
        self.pools
            .get(&culture)
            .map(|pool| pool.generate_name(rng))
            .unwrap_or_else(|| NamePool::western_us().generate_name(rng))
    }

    /// Generate a user ID from a name.
    pub fn generate_user_id(&self, name: &PersonName, index: usize) -> String {
        name.to_user_id(index)
    }

    /// Generate an email from a name.
    pub fn generate_email(&self, name: &PersonName) -> String {
        name.to_email(&self.email_domain)
    }

    /// Create a name generator from a country pack's names configuration.
    pub fn from_country_pack(pack: &crate::country::schema::CountryPack) -> Self {
        let mut pools = HashMap::new();
        let mut distribution = Vec::new();

        for culture_config in &pack.names.cultures {
            // Map culture_id to NameCulture enum; default to WesternUs for unknown.
            let culture = match culture_config.culture_id.as_str() {
                "western_us" | "western" => NameCulture::WesternUs,
                "german" | "deutsch" => NameCulture::German,
                "french" | "français" => NameCulture::French,
                "chinese" | "中文" => NameCulture::Chinese,
                "japanese" | "日本語" => NameCulture::Japanese,
                "indian" | "hindi" => NameCulture::Indian,
                "hispanic" | "latino" | "español" => NameCulture::Hispanic,
                _ => NameCulture::WesternUs,
            };

            let pool = NamePool::from_culture_config(culture_config, culture);
            pools.insert(culture, pool);
            distribution.push((culture, culture_config.weight));
        }

        // Fallback: if no cultures were configured, use default pools.
        if pools.is_empty() {
            return Self::new();
        }

        let email_domain = pack
            .names
            .email_domains
            .first()
            .cloned()
            .unwrap_or_else(|| "company.com".to_string());

        Self {
            pools,
            distribution,
            email_domain,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_name_pool_generation() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let pool = NamePool::western_us();
        let name = pool.generate_name(&mut rng);

        assert!(!name.first_name.is_empty());
        assert!(!name.last_name.is_empty());
        assert_eq!(name.culture, NameCulture::WesternUs);
    }

    #[test]
    fn test_multi_culture_generator() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let generator = MultiCultureNameGenerator::new();

        // Generate multiple names and check distribution
        let mut culture_counts: HashMap<NameCulture, usize> = HashMap::new();
        for _ in 0..100 {
            let name = generator.generate_name(&mut rng);
            *culture_counts.entry(name.culture).or_insert(0) += 1;
        }

        // Should have some diversity
        assert!(culture_counts.len() > 1);
    }

    #[test]
    fn test_user_id_generation() {
        let name = PersonName {
            first_name: "John".to_string(),
            last_name: "Smith".to_string(),
            display_name: "John Smith".to_string(),
            culture: NameCulture::WesternUs,
            is_male: true,
        };

        let user_id = name.to_user_id(42);
        assert_eq!(user_id, "JSMITH042");
    }

    #[test]
    fn test_email_generation() {
        let name = PersonName {
            first_name: "María".to_string(),
            last_name: "García".to_string(),
            display_name: "María García".to_string(),
            culture: NameCulture::Hispanic,
            is_male: false,
        };

        let email = name.to_email("acme.com");
        assert_eq!(email, "mara.garca@acme.com"); // Note: non-ASCII stripped
    }

    #[test]
    fn test_all_culture_pools() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        for culture in NameCulture::all() {
            let pool = NamePool::for_culture(*culture);
            let name = pool.generate_name(&mut rng);
            assert!(!name.first_name.is_empty());
            assert!(!name.last_name.is_empty());
            assert_eq!(name.culture, *culture);
        }
    }
}
