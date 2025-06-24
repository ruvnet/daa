use qudag_network::message::{Message, MessageError, MessageType};

#[test]
fn test_message_creation() {
    let msg = Message::new(
        MessageType::Data,
        vec![1, 2, 3], // sender key
        vec![4, 5, 6], // recipient
        vec![7, 8, 9], // payload
        12345,         // timestamp
    );

    assert!(matches!(msg.msg_type, MessageType::Data));
    assert_eq!(msg.sender, vec![1, 2, 3]);
    assert_eq!(msg.recipient, vec![4, 5, 6]);
    assert_eq!(msg.payload, vec![7, 8, 9]);
    assert_eq!(msg.timestamp, 12345);
}

#[test]
fn test_message_validation() {
    // Valid message
    let msg = Message::new(
        MessageType::Data,
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
        12345,
    );
    assert!(msg.validate().is_ok());

    // Empty sender
    let msg = Message::new(
        MessageType::Data,
        vec![],
        vec![4, 5, 6],
        vec![7, 8, 9],
        12345,
    );
    assert!(matches!(
        msg.validate(),
        Err(MessageError::InvalidFormat(_))
    ));

    // Empty recipient
    let msg = Message::new(
        MessageType::Data,
        vec![1, 2, 3],
        vec![],
        vec![7, 8, 9],
        12345,
    );
    assert!(matches!(
        msg.validate(),
        Err(MessageError::InvalidFormat(_))
    ));

    // Message too large
    let large_payload = vec![0; 2 * 1024 * 1024]; // 2MB
    let msg = Message::new(
        MessageType::Data,
        vec![1, 2, 3],
        vec![4, 5, 6],
        large_payload,
        12345,
    );
    assert!(matches!(
        msg.validate(),
        Err(MessageError::MessageTooLarge { .. })
    ));
}
