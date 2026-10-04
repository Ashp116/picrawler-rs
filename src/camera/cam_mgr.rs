use gstreamer::{self as gst, Device, DeviceMonitor, prelude::*};

pub struct cam_mgr {
    gst_dev_mon: DeviceMonitor,
    main_cam: Option<Device>,
}

impl cam_mgr {
    pub fn new() -> Self {
        gst::init().unwrap();
        gst::log::remove_default_log_function(); // comment this out for debugging

        let monitor = DeviceMonitor::new();
        monitor.add_filter(Some("Video/Source"), None);
        monitor.start().unwrap();

        Self {
            gst_dev_mon: monitor,
            main_cam: None,
        }
    }

    pub fn get_camera(&self, id: &str) -> Option<Device> {
        self.list_cameras().into_iter().find(|cam| cam.display_name() == id)
    }

    pub fn set_main_cam(&mut self, id: &str) {
        if let Some(cam) = self.get_camera(id) {
            self.main_cam = Some(cam);
        } else {
            eprintln!("Invalid camera id: {} ", id);
        }
    }

    pub fn list_cameras(&self) -> Vec<Device> {
        self.gst_dev_mon.devices().into_iter().collect()
    }
}

impl Drop for cam_mgr {
    fn drop(&mut self) {
        self.gst_dev_mon.stop();
    }
}
