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

    // camera -> videoconvert -> videoscale -> appsink (RGB, width x height @ fps)
    pub fn start(&mut self, width: i32, height: i32, fps: i32) -> Result<(), Box<dyn Error>> {
        self.stop();

        let cam = self.main_cam.as_ref().ok_or("no main camera set")?;
        let src = cam.create_element(Some("cam_src"))?;
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
        pipeline.add_many([&src, &convert, &scale, sink.upcast_ref()])?;
        gst::Element::link_many([&src, &convert, &scale, sink.upcast_ref()])?;
        pipeline.set_state(gst::State::Playing)?;

        self.pipeline = Some(pipeline);
        self.sink = Some(sink);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(pipeline) = self.pipeline.take() {
            let _ = pipeline.set_state(gst::State::Null);
        }
        self.sink = None;
    }

    // waits up to timeout_ms for the next frame; None if not running or timed out
    pub fn grab_frame(&self, timeout_ms: u64) -> Option<Frame> {
        let sample = self.sink.as_ref()?.try_pull_sample(gst::ClockTime::from_mseconds(timeout_ms))?;
        let info = gst_video::VideoInfo::from_caps(sample.caps()?).ok()?;
        let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(sample.buffer()?, &info).ok()?;

        let (width, height) = (frame.width(), frame.height());
        let stride = frame.plane_stride()[0] as usize;
        let row_len = width as usize * 3;
        let plane = frame.plane_data(0).ok()?;

        let mut data = Vec::with_capacity(row_len * height as usize);
        for row in plane.chunks(stride).take(height as usize) {
            data.extend_from_slice(&row[..row_len]);
        }

        Some(Frame { width, height, data })
    }
}

impl Drop for cam_mgr {
    fn drop(&mut self) {
        self.stop();
        self.gst_dev_mon.stop();
    }
}
