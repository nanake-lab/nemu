use super::*;
use crate::framing::{FrameConfig, Framed};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
fn frame_config() -> FrameConfig {
    FrameConfig {
        encoding: limits(),
        deadline: Duration::from_millis(50),
    }
}

#[tokio::test]
async fn fragmented_frames_roundtrip_and_preserve_receipt_stages() {
    let (a, b) = tokio::io::duplex(4096);
    let mut sender = Framed::new(a, frame_config()).unwrap();
    let mut receiver = Framed::new(b, frame_config()).unwrap();
    for stage in [
        ReceiptStage::RECEIVED_DURABLE,
        ReceiptStage::CORE_COMMITTED,
        ReceiptStage::FILTERED,
        ReceiptStage::REJECTED,
    ] {
        let receipt = Receipt {
            schema_version: 1,
            stable_source_key: "synthetic-key".into(),
            stage,
            event_id: Some("synthetic-id".into()),
            reason: None,
        };
        sender.write(&receipt).await.unwrap();
        assert_eq!(receiver.read::<Receipt>().await.unwrap(), receipt);
    }
    let (mut a, b) = tokio::io::duplex(4096);
    let bytes = UserText {
        schema_version: 1,
        text: "中/英/日".into(),
    }
    .protobuf_bytes(limits())
    .unwrap();
    let mut wire = (bytes.len() as u32).to_le_bytes().to_vec();
    wire.extend(bytes);
    for byte in wire {
        a.write_all(&[byte]).await.unwrap();
    }
    assert_eq!(
        Framed::new(b, frame_config())
            .unwrap()
            .read::<UserText>()
            .await
            .unwrap()
            .text,
        "中/英/日"
    );
}
#[tokio::test]
async fn oversize_empty_truncated_and_invalid_messages_poison_connection() {
    for body in [
        vec![0, 0, 0, 0],
        vec![255; 4],
        vec![4, 0, 0, 0, 8, 1],
        vec![2, 0, 0, 0, 8, 99],
    ] {
        let (mut a, b) = tokio::io::duplex(128);
        a.write_all(&body).await.unwrap();
        drop(a);
        let mut framed = Framed::new(b, frame_config()).unwrap();
        assert!(framed.read::<UserText>().await.is_err());
        assert!(matches!(
            framed.read::<UserText>().await,
            Err(Error::Rejected("connection is closed"))
        ));
    }
}
#[tokio::test]
async fn slow_header_body_writer_and_cancelled_read_are_bounded() {
    let (mut a, b) = tokio::io::duplex(128);
    let mut framed = Framed::new(b, frame_config()).unwrap();
    a.write_all(&[3, 0]).await.unwrap();
    assert!(matches!(
        framed.read::<UserText>().await,
        Err(Error::Timeout)
    ));
    let (mut a, b) = tokio::io::duplex(128);
    a.write_all(&[20, 0, 0, 0, 8, 1]).await.unwrap();
    let mut framed = Framed::new(b, frame_config()).unwrap();
    assert!(matches!(
        framed.read::<UserText>().await,
        Err(Error::Timeout)
    ));
    let (a, _b) = tokio::io::duplex(1);
    let mut writer = Framed::new(a, frame_config()).unwrap();
    assert!(matches!(
        writer
            .write(&UserText {
                schema_version: 1,
                text: "synthetic".into()
            })
            .await,
        Err(Error::Timeout)
    ));
    let (_a, b) = tokio::io::duplex(128);
    let mut reader = Framed::new(b, frame_config()).unwrap();
    assert!(
        tokio::time::timeout(Duration::from_millis(5), reader.read::<UserText>())
            .await
            .is_err()
    );
    assert!(matches!(
        reader.read::<UserText>().await,
        Err(Error::Rejected("connection is closed"))
    ));
}
#[tokio::test]
async fn wire_bytes_are_length_prefixed_protobuf() {
    let (a, mut b) = tokio::io::duplex(128);
    let value = UserText {
        schema_version: 1,
        text: "abc".into(),
    };
    let expected = value.protobuf_bytes(limits()).unwrap();
    Framed::new(a, frame_config())
        .unwrap()
        .write(&value)
        .await
        .unwrap();
    assert_eq!(b.read_u32_le().await.unwrap() as usize, expected.len());
    let mut bytes = vec![0; expected.len()];
    b.read_exact(&mut bytes).await.unwrap();
    assert_eq!(bytes, expected);
}
