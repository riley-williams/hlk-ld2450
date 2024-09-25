use embedded_io_async::{Read, ReadExactError, Write};

const COMMAND_HEADER: [u8; 4] = [0xFD, 0xFC, 0xFB, 0xFA];
const END_OF_FRAME: [u8; 4] = [0x04, 0x03, 0x02, 0x01];
const SEND_COMMAND_WORD: [u8; 2] = [0x01, 0x00];

pub(crate) async fn enter_config_mode<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command_data(writer, 0x01, 0x01).await
}

pub(crate) async fn exit_config_mode<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0xFE).await
}

pub(crate) async fn set_single_target_tracking<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0x80).await
}

pub(crate) async fn set_multi_target_tracking<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0x90).await
}

pub(crate) async fn get_target_tracking_mode<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0x91).await
}

pub(crate) async fn get_firmware_version<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0xA0).await
}

pub(crate) async fn set_baud_rate<W: Write>(
    writer: &mut W,
    baud_rate: crate::BaudRate,
) -> Result<(), W::Error> {
    write_command_data(writer, 0xA1, baud_rate.byte_repr()).await
}

pub(crate) async fn factory_restore<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0xA2).await
}

pub(crate) async fn restart<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command(writer, 0xA3).await
}

pub(crate) async fn set_bluetooth_enabled<W: Write>(
    writer: &mut W,
    enabled: bool,
) -> Result<(), W::Error> {
    let data = if enabled { 0x01 } else { 0x00 };
    write_command_data(writer, 0xA4, data).await
}

pub(crate) async fn get_mac_address<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command_data(writer, 0xA5, 0x01).await
}

pub(crate) async fn get_zone_filtering<W: Write>(writer: &mut W) -> Result<(), W::Error> {
    write_command_data(writer, 0xC1, 0x01).await
}

pub(crate) async fn set_zone_filtering<W: Write>(
    writer: &mut W,
    zone_filtering: &[u8],
) -> Result<(), W::Error> {
    let msg_len = 2 + zone_filtering.len() as u16;
    writer.write_all(&COMMAND_HEADER).await?;
    writer.write_all(&msg_len.to_le_bytes()).await?;
    writer.write_all(&0xC2u16.to_le_bytes()).await?;
    writer.write_all(zone_filtering).await?;
    writer.write_all(&END_OF_FRAME).await
}

pub(crate) async fn write_command<W: Write>(writer: &mut W, command: u16) -> Result<(), W::Error> {
    writer.write_all(&COMMAND_HEADER).await?;
    writer.write_all(&2u16.to_le_bytes()).await?;
    writer.write_all(&command.to_le_bytes()).await?;
    writer.write_all(&END_OF_FRAME).await
}

pub(crate) async fn write_command_data<W: Write>(
    writer: &mut W,
    command: u16,
    data: u16,
) -> Result<(), W::Error> {
    writer.write_all(&COMMAND_HEADER).await?;
    writer.write_all(&4u16.to_le_bytes()).await?;
    writer.write_all(&command.to_le_bytes()).await?;
    writer.write_all(&data.to_le_bytes()).await?;
    writer.write_all(&END_OF_FRAME).await
}

pub(crate) async fn ack<const N: usize, R: Read>(
    reader: &mut R,
    buf: &mut [u8; N],
) -> Result<(), ReadExactError<R::Error>> {
    let mut temp = [0; 4];
    reader.read_exact(&mut temp).await?;
    if temp != COMMAND_HEADER {
        // TODO: real error
        return Err(ReadExactError::UnexpectedEof);
    }

    reader.read_exact(&mut temp[0..2]).await?;
    let length = u16::from_le_bytes([temp[0], temp[1]]);
    if length != N as u16 {
        // TODO: real error
        return Err(ReadExactError::UnexpectedEof);
    }

    reader.read_exact(&mut buf[..]).await?;

    // EOF
    reader.read_exact(&mut temp).await?;
    if temp != END_OF_FRAME {
        // TODO: real error
        return Err(ReadExactError::UnexpectedEof);
    }

    Ok(())
}
