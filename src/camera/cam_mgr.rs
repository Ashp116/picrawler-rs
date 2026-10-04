use std::error::Error;

use gstreamer::{self as gst, Device, DeviceMonitor, prelude::*};
use gstreamer_app::AppSink;
use gstreamer_video::{self as gst_video, prelude::*};

// one tightly packed RGB frame (3 bytes per pixel, no row padding)
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct cam_mgr {
    gst_dev_mon: DeviceMonitor,
    main_cam: Option<Device>,
    pipeline: Option<gst::Pipeline>,
    sink: Option<AppSink>,
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
            pipeline: None,
            sink: None,
        }
    }

    pub fn get_camera(&self, id: &str) -> Option<Device> {
        self.list_cameras().into_iter().find(|cam| cam.display_name() == id)
    }

    pub fn set_main_cam(&mut self, id: &str) {
        if let Some(cam) = self.get_camera(id) {
            self.stop();
            self.main_cam = Some(cam);
        } else {
            eprintln!("Invalid camera id: {} ", id);
        }
    }

    pub fn list_cameras(&self) -> Vec<Device> {
        self.gst_dev_mon.devices().into_iter().collect()
    }

    // on the pi the v4l2 provider also lists raw csi/codec/isp nodes (rp1-cfe, unicam,
    // bcm2835-*) as video sources; those can't stream on their own, only libcamera can
    pub fn default_camera(&self) -> Option<Device> {
        self.list_cameras().into_iter().find(|cam| {
            cam.create_element(None)
                .ok()
                .and_then(|e| e.factory())
                .is_some_and(|f| f.name() == "libcamerasrc")
        })
    }

    // camera -> videorate -> videoconvert -> videoscale -> appsink (RGB, width x height @ fps)
    pub fn start(&mut self, width: i32, height: i32, fps: i32) -> Result<(), Box<dyn Error>> {
        self.stop();

        // with no main camera set, let libcamerasrc pick the first sensor itself
        let src = match &self.main_cam {
            Some(cam) => cam.create_element(Some("cam_src"))?,
            None => gst::ElementFactory::make("libcamerasrc")
                .name("cam_src")
                .build()
                .map_err(|_| "libcamerasrc not found, install gstreamer1.0-libcamera")?,
        };
        let rate = gst::ElementFactory::make("videorate").build()?;
        let convert = gst::ElementFactory::make("videoconvert").build()?;
        let scale = gst::ElementFactory::make("videoscale").build()?;

        let caps = gst_video::VideoCapsBuilder::new()
            .format(gst_video::VideoFormat::Rgb)
            .width(width)
            .height(height)
            .framerate(gst::Fraction::new(fps, 1))
            .build();
        // keep only the newest frame so a slow reader never sees stale video
        let sink = AppSink::builder()
            .caps(&caps)
            .max_buffers(1)
            .drop(true)
            .sync(false)
            .build();

        let pipeline = gst::Pipeline::new();
        pipeline.add_many([&src, &rate, &convert, &scale, sink.upcast_ref()])?;
        gst::Element::link_many([&src, &rate, &convert, &scale, sink.upcast_ref()])?;

        self.pipeline = Some(pipeline.clone());
        self.sink = Some(sink);

        // wait for the camera to actually come up so negotiation errors surface here
        let started = pipeline.set_state(gst::State::Playing).is_ok()
            && pipeline.state(gst::ClockTime::from_seconds(5)).0.is_ok();
        if let Some(err) = self.bus_error() {
            self.stop();
            return Err(err.into());
        }
        if !started {
            self.stop();
            return Err("pipeline failed to reach PLAYING".into());
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(pipeline) = self.pipeline.take() {
            let _ = pipeline.set_state(gst::State::Null);
        }
        self.sink = None;
    }

    // pops the first pending error off the pipeline bus, if any
    fn bus_error(&self) -> Option<String> {
        let bus = self.pipeline.as_ref()?.bus()?;
        let msg = bus.pop_filtered(&[gst::MessageType::Error])?;
        let gst::MessageView::Error(err) = msg.view() else { return None };
        Some(format!(
            "{} (from {}): {}",
            err.error(),
            msg.src().map(|s| s.path_string().to_string()).unwrap_or_default(),
            err.debug().unwrap_or_default()
        ))
    }

    // waits up to timeout_ms for the next frame
    pub fn grab_frame(&self, timeout_ms: u64) -> Result<Frame, String> {
        let sink = self.sink.as_ref().ok_or("camera not started")?;
        let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_mseconds(timeout_ms)) else {
            return Err(self.bus_error().unwrap_or_else(|| format!("no frame within {}ms", timeout_ms)));
        };

        let caps = sample.caps().ok_or("sample has no caps")?;
        let buffer = sample.buffer().ok_or("sample has no buffer")?;
        let info = gst_video::VideoInfo::from_caps(caps).map_err(|e| e.to_string())?;
        let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info).map_err(|e| e.to_string())?;

        let (width, height) = (frame.width(), frame.height());
        let stride = frame.plane_stride()[0] as usize;
        let row_len = width as usize * 3;
        let plane = frame.plane_data(0).map_err(|e| e.to_string())?;

        let mut data = Vec::with_capacity(row_len * height as usize);
        for row in plane.chunks(stride).take(height as usize) {
            data.extend_from_slice(&row[..row_len]);
        }

        Ok(Frame { width, height, data })
    }
}

impl Drop for cam_mgr {
    fn drop(&mut self) {
        self.stop();
        self.gst_dev_mon.stop();
    }
}
