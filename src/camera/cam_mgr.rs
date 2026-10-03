use libcamera::{camera::Camera, camera_manager::{CameraList, CameraManager}};

pub struct cam_mgr {
    lib_cam_mgr: CameraManager,
    main_cam: Option<Camera<'static>>,
}

impl cam_mgr {
    pub fn new() -> Self {
        Self {
            lib_cam_mgr: CameraManager::new().unwrap(),
            main_cam: None,
        }
    }

    pub fn set_main_cam(&mut self, id: &str) {
        self.main_cam = self.lib_cam_mgr.get(id);
    }

    pub fn list_cameras(&self) -> CameraList {
        self.lib_cam_mgr.cameras()
    }  
}