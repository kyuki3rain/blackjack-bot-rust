use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Betting,
    Dealing,
    Playing(usize),
    DealerTurn,
    End,
}

// FromStrトレイトの実装
impl FromStr for Status {
    type Err = &'static str; // エラー時の型

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Betting" => Ok(Status::Betting),
            "Dealing" => Ok(Status::Dealing),
            "DealerTurn" => Ok(Status::DealerTurn),
            "End" => Ok(Status::End),
            _ => {
                if s.starts_with("Playing ") {
                    let index = s
                        .split_whitespace()
                        .nth(1)
                        .ok_or("Playing index not found")?
                        .parse()
                        .map_err(|_| "Playing index is not a number")?;
                    Ok(Status::Playing(index))
                } else {
                    Err("Invalid status")
                }
            }
        }
    }
}

// ToStringトレイトの実装
impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Betting => "Betting".to_string(),
            Status::Dealing => "Dealing".to_string(),
            Status::DealerTurn => "DealerTurn".to_string(),
            Status::End => "End".to_string(),
            Status::Playing(i) => format!("Playing {}", i),
        }
    }
}
