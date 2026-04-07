/// Standard HID Keyboard SDP service record XML.
/// This makes the device appear as "Standard Bluetooth Keyboard" to remote hosts.
/// The descriptor is passed verbatim to BlueZ during profile registration.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub const HID_SDP_RECORD: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<record>
  <!-- ServiceClassIDList: HumanInterfaceDevice -->
  <attribute id="0x0001">
    <sequence>
      <uuid value="0x1124"/>
    </sequence>
  </attribute>

  <!-- ProtocolDescriptorList: L2CAP PSM=17 (HID Control) + HIDP -->
  <attribute id="0x0004">
    <sequence>
      <sequence>
        <uuid value="0x0100"/>
        <uint16 value="0x0011"/>
      </sequence>
      <sequence>
        <uuid value="0x0011"/>
      </sequence>
    </sequence>
  </attribute>

  <!-- BrowseGroupList -->
  <attribute id="0x0005">
    <sequence>
      <uuid value="0x1002"/>
    </sequence>
  </attribute>

  <!-- AdditionalProtocolDescriptorList: L2CAP PSM=19 (HID Interrupt) + HIDP -->
  <attribute id="0x000d">
    <sequence>
      <sequence>
        <sequence>
          <uuid value="0x0100"/>
          <uint16 value="0x0013"/>
        </sequence>
        <sequence>
          <uuid value="0x0011"/>
        </sequence>
      </sequence>
    </sequence>
  </attribute>

  <!-- ServiceName -->
  <attribute id="0x0100">
    <text value="Standard Bluetooth Keyboard+Mouse"/>
  </attribute>

  <!-- ServiceDescription -->
  <attribute id="0x0101">
    <text value="Keyboard/Mouse Combo"/>
  </attribute>

  <!-- ProviderName -->
  <attribute id="0x0102">
    <text value="ccpay-human-commander"/>
  </attribute>

  <!-- HIDParserVersion -->
  <attribute id="0x0201">
    <uint16 value="0x0111"/>
  </attribute>

  <!-- HIDDeviceSubclass: Keyboard+Mouse combo (0xC0) -->
  <attribute id="0x0202">
    <uint8 value="0xC0"/>
  </attribute>

  <!-- HIDCountryCode -->
  <attribute id="0x0203">
    <uint8 value="0x00"/>
  </attribute>

  <!-- HIDVirtualCable -->
  <attribute id="0x0204">
    <boolean value="false"/>
  </attribute>

  <!-- HIDReconnectInitiate -->
  <attribute id="0x0205">
    <boolean value="false"/>
  </attribute>

  <!-- HIDDescriptorList: standard boot keyboard report descriptor -->
  <attribute id="0x0206">
    <sequence>
      <sequence>
        <uint8 value="0x22"/>
        <text encoding="hex" value="05010906a101850175019508050719e029e715002501810295017508810395057501050819012905910295017503910395067508150026ff000507190029ff8100c0050c0901a101850219002aff03150026ff03751095018102c005010902a10185030901a10005090901190329031500250175018102009501075058103005010930093116018026ff7f751095028106050109381581257f750895018106c0c0"/>
      </sequence>
    </sequence>
  </attribute>

  <!-- HIDLangIDBaseList -->
  <attribute id="0x0207">
    <sequence>
      <sequence>
        <uint16 value="0x0409"/>
        <uint16 value="0x0100"/>
      </sequence>
    </sequence>
  </attribute>

  <!-- HIDBootDevice -->
  <attribute id="0x020e">
    <boolean value="true"/>
  </attribute>

  <!-- HIDSupervisionTimeout -->
  <attribute id="0x020c">
    <uint16 value="0x0c80"/>
  </attribute>

  <!-- HIDNormallyConnectable -->
  <attribute id="0x020d">
    <boolean value="false"/>
  </attribute>

  <!-- HIDProfileVersion -->
  <attribute id="0x020b">
    <uint16 value="0x0100"/>
  </attribute>
</record>"#;
