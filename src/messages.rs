use std::{future::Future, pin::Pin};

use prost::{bytes::Buf, Message};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{
    api::*,
    result::Result,
    variant::{VarintRead, VarintWrite},
};

macro_rules! esphome_messages {
    ($($id:expr => $name:ident),* $(,)?) => {
        #[derive(Clone, PartialEq, Debug)]
        #[derive(Default)]
        pub enum ESPHomeMessage {
            #[default]
            None,
            Unknown(u32),
            $($name($name),)*
        }

        impl ESPHomeMessage {
            pub fn id(&self) -> u32 {
                match self {
                    $(ESPHomeMessage::$name(_) => $id,)*
                    ESPHomeMessage::None => 0,
                    ESPHomeMessage::Unknown(id) => *id,
                }
            }

            pub fn encode(&self) -> Result<Vec<u8>> {
                let mut bytes = Vec::new();
                match self {
                    $(ESPHomeMessage::$name(msg) => msg.encode(&mut bytes)?,)*
                    ESPHomeMessage::None => return Err("Cannot encode None message".into()),
                    ESPHomeMessage::Unknown(_) => return Err("Cannot encode unknown message type".into()),
                }
                Ok(bytes)
            }

            pub(crate) fn from_buffer<B>(
                message_id: u32,
                buf: B
            ) -> Result<ESPHomeMessage>
            where
                B: Buf + std::marker::Unpin,
            {
                match message_id {
                    $($id => Ok(ESPHomeMessage::$name(<$name>::decode(buf)?)),)*
                    0 => Ok(ESPHomeMessage::None),
                    _ => Ok(ESPHomeMessage::Unknown(message_id)),
                }
            }
        }
    };
}

esphome_messages! {
    1 => HelloRequest,
    2 => HelloResponse,
    3 => ConnectRequest,
    4 => ConnectResponse,
    5 => DisconnectRequest,
    6 => DisconnectResponse,
    7 => PingRequest,
    8 => PingResponse,
    9 => DeviceInfoRequest,
    10 => DeviceInfoResponse,
    11 => ListEntitiesRequest,
    19 => ListEntitiesDoneResponse,
    20 => SubscribeStatesRequest,
    12 => ListEntitiesBinarySensorResponse,
    21 => BinarySensorStateResponse,
    13 => ListEntitiesCoverResponse,
    22 => CoverStateResponse,
    30 => CoverCommandRequest,
    14 => ListEntitiesFanResponse,
    23 => FanStateResponse,
    31 => FanCommandRequest,
    15 => ListEntitiesLightResponse,
    24 => LightStateResponse,
    32 => LightCommandRequest,
    16 => ListEntitiesSensorResponse,
    25 => SensorStateResponse,
    17 => ListEntitiesSwitchResponse,
    26 => SwitchStateResponse,
    33 => SwitchCommandRequest,
    18 => ListEntitiesTextSensorResponse,
    27 => TextSensorStateResponse,
    28 => SubscribeLogsRequest,
    29 => SubscribeLogsResponse,
    34 => SubscribeHomeassistantServicesRequest,
    35 => HomeassistantServiceResponse,
    38 => SubscribeHomeAssistantStatesRequest,
    39 => SubscribeHomeAssistantStateResponse,
    40 => HomeAssistantStateResponse,
    36 => GetTimeRequest,
    37 => GetTimeResponse,
    41 => ListEntitiesServicesResponse,
    42 => ExecuteServiceRequest,
    43 => ListEntitiesCameraResponse,
    44 => CameraImageResponse,
    45 => CameraImageRequest,
    46 => ListEntitiesClimateResponse,
    47 => ClimateStateResponse,
    48 => ClimateCommandRequest,
    49 => ListEntitiesNumberResponse,
    50 => NumberStateResponse,
    51 => NumberCommandRequest,
    52 => ListEntitiesSelectResponse,
    53 => SelectStateResponse,
    54 => SelectCommandRequest,
    58 => ListEntitiesLockResponse,
    59 => LockStateResponse,
    60 => LockCommandRequest,
    61 => ListEntitiesButtonResponse,
    62 => ButtonCommandRequest,
    63 => ListEntitiesMediaPlayerResponse,
    64 => MediaPlayerStateResponse,
    65 => MediaPlayerCommandRequest,
    66 => SubscribeBluetoothLeAdvertisementsRequest,
    67 => BluetoothLeAdvertisementResponse,
    93 => BluetoothLeRawAdvertisementsResponse,
    68 => BluetoothDeviceRequest,
    69 => BluetoothDeviceConnectionResponse,
    70 => BluetoothGattGetServicesRequest,
    71 => BluetoothGattGetServicesResponse,
    72 => BluetoothGattGetServicesDoneResponse,
    73 => BluetoothGattReadRequest,
    74 => BluetoothGattReadResponse,
    75 => BluetoothGattWriteRequest,
    76 => BluetoothGattReadDescriptorRequest,
    77 => BluetoothGattWriteDescriptorRequest,
    78 => BluetoothGattNotifyRequest,
    79 => BluetoothGattNotifyDataResponse,
    80 => SubscribeBluetoothConnectionsFreeRequest,
    81 => BluetoothConnectionsFreeResponse,
    82 => BluetoothGattErrorResponse,
    83 => BluetoothGattWriteResponse,
    84 => BluetoothGattNotifyResponse,
    85 => BluetoothDevicePairingResponse,
    86 => BluetoothDeviceUnpairingResponse,
    87 => UnsubscribeBluetoothLeAdvertisementsRequest,
    88 => BluetoothDeviceClearCacheResponse,
    89 => SubscribeVoiceAssistantRequest,
    90 => VoiceAssistantRequest,
    91 => VoiceAssistantResponse,
    92 => VoiceAssistantEventResponse,
    94 => ListEntitiesAlarmControlPanelResponse,
    95 => AlarmControlPanelStateResponse,
    96 => AlarmControlPanelCommandRequest,
    97 => ListEntitiesTextResponse,
    98 => TextStateResponse,
    99 => TextCommandRequest,
}

pub trait ESPHomeMessageWrite {
    fn write_esphome_message(
        &mut self,
        message: ESPHomeMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

pub trait ESPHOmeMessageRead {
    fn read_esphome_message(&mut self) -> Pin<Box<dyn Future<Output = Result<ESPHomeMessage>> + Send + '_>>;
}

impl<W> ESPHomeMessageWrite for W
where
    W: AsyncWrite + Unpin + Sized + VarintWrite + Send,
{
    fn write_esphome_message(
        &mut self,
        message: ESPHomeMessage,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let message_bytes = message.encode()?;
            self.write_u8(0).await?;
            self.write_varint(message_bytes.len() as u32).await?;
            self.write_varint(message.id()).await?;
            self.write_all(&message_bytes).await?;
            self.flush().await?;

            Ok(())
        })
    }
}

impl<R> ESPHOmeMessageRead for R
where
    R: AsyncRead + Unpin + Sized + VarintRead + Send,
{
    fn read_esphome_message(&mut self) -> Pin<Box<dyn Future<Output = Result<ESPHomeMessage>> + Send + '_>> {
        Box::pin(async move {
            let magic_byte = self.read_u8().await?;
            if magic_byte != 0 {
                return Err("invalid first byte of the message".into());
            }

            let len = self.read_varint().await? as usize;
            let message_id = self.read_varint().await?;

            let mut buf = vec![0u8; len];
            self.read_exact(&mut buf).await?;

            ESPHomeMessage::from_buffer(message_id, &*buf)
        })
    }
}
