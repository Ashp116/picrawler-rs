use libcamera::{camera::Camera, camera_manager::{CameraList, CameraManager}, logging::{LoggingLevel::Error, LoggingTarget, log_set_target}};

pub struct cam_mgr {
    lib_cam_mgr: CameraManager,
    main_cam: Option<Camera<'static>>,
}

impl cam_mgr {
    pub fn new() -> Self {
        let cam = CameraManager::new().unwrap();
        log_set_target(LoggingTarget::None); // change this for debugging
        cam.log_set_level("*", Error);

        Self {
            lib_cam_mgr: cam,
            main_cam: None,
        }
    }

    pub fn get_camera(&self, id: &str) -> Option<Camera<'static>> {
        self.lib_cam_mgr.get(id)
    }

    pub fn set_main_cam(&mut self, id: &str) {
        if let Some(cam) = self.get_camera(id) {
            self.main_cam = Some(cam);
        } else {
            eprintln!("Invalid camera id: {} ", id);
        }
    }

    pub fn list_cameras(&self) -> CameraList {
        self.lib_cam_mgr.cameras()
    }  
}