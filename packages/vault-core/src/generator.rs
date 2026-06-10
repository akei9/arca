use rand::prelude::{IndexedRandom, SliceRandom};

const UPPERCASE: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const LOWERCASE: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];
const DIGITS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const SYMBOLS: &[char] = &[
    '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', ']', '{', '}', ':',
    ';', ',', '.', '?',
];
const AMBIGUOUS: &[char] = &['0', 'O', '1', 'l', 'I'];
const MIN_LENGTH: usize = 8;
const MAX_LENGTH: usize = 128;

pub enum GeneratorMode {
    Random,
    Passphrase,
}

pub struct GeneratorConfig {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
    pub mode: GeneratorMode,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            length: 24,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: false,
            mode: GeneratorMode::Random,
        }
    }
}

pub fn generate_password(config: &GeneratorConfig) -> String {
    match config.mode {
        GeneratorMode::Random | GeneratorMode::Passphrase => generate_random_password(config),
    }
}

pub fn calculate_entropy(password: &str, config: &GeneratorConfig) -> f64 {
    let charset_size = build_charset(config).len();

    if charset_size == 0 || password.is_empty() {
        return 0.0;
    }

    password.chars().count() as f64 * (charset_size as f64).log2()
}

fn generate_random_password(config: &GeneratorConfig) -> String {
    let required_sets = enabled_charsets(config);
    let full_charset = build_charset(config);

    if required_sets.is_empty() || full_charset.is_empty() {
        return String::new();
    }

    let length = config.length.clamp(MIN_LENGTH, MAX_LENGTH);
    let mut rng = rand::rng();
    let mut chars = Vec::with_capacity(length);

    for charset in &required_sets {
        if let Some(character) = charset.choose(&mut rng) {
            chars.push(*character);
        }
    }

    while chars.len() < length {
        if let Some(character) = full_charset.choose(&mut rng) {
            chars.push(*character);
        }
    }

    chars.shuffle(&mut rng);
    chars.into_iter().collect()
}

fn enabled_charsets(config: &GeneratorConfig) -> Vec<Vec<char>> {
    let mut sets = Vec::new();

    if config.uppercase {
        sets.push(filter_ambiguous(UPPERCASE, config.exclude_ambiguous));
    }
    if config.lowercase {
        sets.push(filter_ambiguous(LOWERCASE, config.exclude_ambiguous));
    }
    if config.digits {
        sets.push(filter_ambiguous(DIGITS, config.exclude_ambiguous));
    }
    if config.symbols {
        sets.push(filter_ambiguous(SYMBOLS, config.exclude_ambiguous));
    }

    sets.into_iter().filter(|set| !set.is_empty()).collect()
}

fn build_charset(config: &GeneratorConfig) -> Vec<char> {
    enabled_charsets(config).into_iter().flatten().collect()
}

fn filter_ambiguous(charset: &[char], exclude_ambiguous: bool) -> Vec<char> {
    if exclude_ambiguous {
        charset
            .iter()
            .copied()
            .filter(|character| !AMBIGUOUS.contains(character))
            .collect()
    } else {
        charset.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{calculate_entropy, generate_password, GeneratorConfig};

    #[test]
    fn test_generated_password_length() {
        let config = GeneratorConfig {
            length: 32,
            ..GeneratorConfig::default()
        };

        let password = generate_password(&config);

        assert_eq!(password.chars().count(), 32);
    }

    #[test]
    fn test_generated_password_contains_required_chars() {
        let config = GeneratorConfig {
            length: 32,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
            ..GeneratorConfig::default()
        };

        let password = generate_password(&config);

        assert!(password
            .chars()
            .any(|character| character.is_ascii_uppercase()));
        assert!(password
            .chars()
            .any(|character| character.is_ascii_lowercase()));
        assert!(password.chars().any(|character| character.is_ascii_digit()));
        assert!(password
            .chars()
            .any(|character| !character.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_entropy_calculation() {
        let config = GeneratorConfig {
            uppercase: false,
            lowercase: false,
            digits: true,
            symbols: false,
            ..GeneratorConfig::default()
        };

        let entropy = calculate_entropy("12345678", &config);

        assert!((entropy - 26.575_424_759).abs() < 0.000_001);
    }

    #[test]
    fn test_unique_passwords() {
        let config = GeneratorConfig::default();
        let mut passwords = HashSet::new();

        for _ in 0..100 {
            passwords.insert(generate_password(&config));
        }

        assert_eq!(passwords.len(), 100);
    }

    #[test]
    fn excludes_ambiguous_characters() {
        let config = GeneratorConfig {
            length: 128,
            exclude_ambiguous: true,
            ..GeneratorConfig::default()
        };

        let password = generate_password(&config);

        assert!(!password
            .chars()
            .any(|character| "0O1lI".contains(character)));
    }
}
