#![no_std]

pub mod state;

/// Messages the host sends to the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    Ping,
    Time { hours: u8, minutes: u8, seconds: u8 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_compare_by_value() {
        let a = Message::Time { hours: 9, minutes: 30, seconds: 0 };
        let b = Message::Time { hours: 9, minutes: 30, seconds: 0 };
        assert_eq!(a, b);
        assert_ne!(a, Message::Ping);
    }
}
