#[cfg(test)]
mod tests {
    extern crate redisish;
    use self::redisish::{parse, Command, Error};

    #[test]
    fn test_retrieve() {
        let line = "RETRIEVE\n";
        let result = parse(line);
        assert_eq!(result, Ok(Command::Retrieve));
    }

    #[test]
    fn test_publish() {
        let line = "PUBLISH TestMessage\n";
        let result = parse(line);
        assert_eq!(result, Ok(Command::Publish("TestMessage".into())));
    }

    #[test]
    fn test_empty_string() {
        let line = "";
        let result = parse(line);
        assert_eq!(result, Err(Error::IncompleteMessage));
    }

    #[test]
    fn test_missing_newline() {
        let line = "FooBar";
        let result = parse(line);
        assert_eq!(result, Err(Error::IncompleteMessage));
    }

    #[test]
    fn test_retrieve_with_payload() {
        let line = "RETRIEVE payload\n";
        let result = parse(line);
        assert_eq!(result, Err(Error::UnexpectedPayload));
    }

    #[test]
    fn test_publish_without_payload() {
        let line = "PUBLISH\n";
        let result = parse(line);
        assert_eq!(result, Err(Error::MissingPayload));
    }
}
