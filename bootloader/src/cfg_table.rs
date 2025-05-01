use core::format_args;

use crate::makros;
use r_efi::base::Guid;

#[derive(Debug)]
pub struct CfgTableType(r_efi::efi::Guid);

//Constructor that allows us to use .into() on any uefi::Guid to convert to CfgTableType
impl From<Guid> for CfgTableType {
    fn from(guid: Guid) -> Self {
        Self(guid)
    }
}

/*
 * This implements a Display trait that will output the human-readable name for any recognized GUIDs
 * while falling back to the Display implementation in uefi::Guid, for unrecognized ones.
 */

// i am actually to lazy to create a Bitmap to make this block better, this works just fine
impl core::fmt::Display for CfgTableType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        match self.0 {
            r_efi::system::ACPI_20_TABLE_GUID => f.write_str("ACPI 2.0 Table"),
            r_efi::system::ACPI_10_TABLE_GUID => f.write_str("ACPI 1.0 Table"),
            r_efi::system::MEMORY_ATTRIBUTES_TABLE_GUID => f.write_str("Memory Attributes Table"),
            r_efi::system::RT_PROPERTIES_TABLE_GUID => f.write_str("Runtime Properties Table"),
            r_efi::system::MPS_TABLE_GUID => f.write_str("MultiProcessor Specification Table"),
            r_efi::system::PROPERTIES_TABLE_GUID => f.write_str("Properties Table"),
            r_efi::system::SMBIOS3_TABLE_GUID => f.write_str("SMBIOS 3.0 Table"),
            r_efi::system::SMBIOS_TABLE_GUID => f.write_str("SMBIOS Table"),
            r_efi::system::DTB_TABLE_GUID => f.write_str("Device Tree Blob Table"),
            r_efi::system::SAL_SYSTEM_TABLE_GUID => f.write_str("Sal System"),
            r_efi::system::JSON_CONFIG_DATA_TABLE_GUID => f.write_str("JSON Configuration Table"),
            r_efi::system::JSON_CAPSULE_DATA_TABLE_GUID => f.write_str("JSON Capsule Table"),
            r_efi::system::JSON_CAPSULE_RESULT_TABLE_GUID => {
                f.write_str("JSON Capsule Result Table")
            }
            r_efi::system::CONFORMANCE_PROFILES_TABLE_GUID => {
                f.write_str("EFI specification conformance")
            }
            r_efi::system::CONFORMANCE_PROFILES_UEFI_SPEC_GUID => {
                f.write_str("Conformance Profiles")
            }
            r_efi::system::HARDWARE_ERROR_VARIABLE_GUID => f.write_str("Hardware Error"),
            _ => write!(f, "GUID: {:?}", self.0),
        }
    }
}
