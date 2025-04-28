use r_efi::efi::{self, Boolean, BootServices, Status};
pub fn imagehandler(bs: &BootServices, image_handler: &efi::Handle) -> efi::Handle {
    let mut new_image_handle: efi::Handle = core::ptr::null_mut();

    let status = {
        (bs.load_image)(
            Boolean::FALSE, // Load image into memory, not into the boot device
            *image_handler,
            core::ptr::null_mut(), // Device handle (null if we want to load from the default device) -> optional
            core::ptr::null_mut(),
            0,
            &mut new_image_handle as &mut _, //OUT parameter: gets the new image handle)
        )
    };

    if status != Status::SUCCESS {
        println!("Error loading image: {:?}\r", status);
    }

    new_image_handle
}
