use std::time::Duration;

use crate::Result;
use nalgebra::Rotation3;
use opencv::{
    aruco::{
        detect_markers, estimate_pose_single_markers_def, get_predefined_dictionary,
        DetectorParameters, DetectorParametersTrait, Dictionary, PREDEFINED_DICTIONARY_NAME,
    },
    calib3d::rodrigues_def,
    core::{no_array, Mat, MatTraitConstManual, Point2f, Ptr, ToInputArray, Vec3d, Vector},
};

#[derive(Debug, Clone, Copy)]
pub struct Aruco {
    pub id: i32,
    pub corners: [[f32; 2];4],
    pub trans: [f64; 3],
    pub euler_angles: [f64; 3],
    pub time_stamp: Duration,
}

pub struct ArucoIntrinsic {
    marker_length: f32,
    dictionary: PREDEFINED_DICTIONARY_NAME,
}

impl ArucoIntrinsic {
    pub fn new_with_marker_length(marker_length: f32) -> Self {
        Self {
            marker_length,
            dictionary: PREDEFINED_DICTIONARY_NAME::DICT_4X4_50,
        }
    }
}

pub struct CameraIntrinsic {
    pub cx: f64,
    pub cy: f64,
    pub fx: f64,
    pub fy: f64,
}

//  inline formula
pub enum CameraDistortion {
    // k1, k2, p1, p2
    Distortion4([f64; 4]),

    // k1, k2, p1, p2, k3
    Distortion5([f64; 5]),

    // k1, k2, p1, p2, k3, k4, k5, k6
    Distortion8([f64; 8]),

    // k1, k2, p1, p2, k3, k4, k5, k6, s1, s2, s3, s4
    Distortion12([f64; 12]),
}

impl CameraDistortion {
    fn as_slice(&self) -> &[f64] {
        use CameraDistortion::*;
        match self {
            Distortion4(d) => d.as_slice(),
            Distortion5(d) => d.as_slice(),
            Distortion8(d) => d.as_slice(),
            Distortion12(d) => d.as_slice(),
        }
    }
}

impl CameraDistortion {
    pub fn from_4_params(k1: f64, k2: f64, p1: f64, p2: f64) -> Self {
        Self::Distortion4([k1, k2, p1, p2])
    }

    pub fn from_5_params(k1: f64, k2: f64, p1: f64, p2: f64, k3: f64) -> Self {
        Self::Distortion5([k1, k2, p1, p2, k3])
    }
}

pub struct ArucoFinderSetting {
    pub aruco_intrinsic: ArucoIntrinsic,
    pub camera_intrinsic: CameraIntrinsic,
    pub camera_distortion: CameraDistortion,
}

pub struct ArucoFinder {
    dictionary: Ptr<Dictionary>,
    detector_paramter: Ptr<DetectorParameters>,
    setting: ArucoFinderSetting,
    camera_matrix: Mat,
    dist_coeffs: Vector<f64>,
}

impl ArucoFinder {
    pub fn new(setting: ArucoFinderSetting) -> Self {
        let dictionary = get_predefined_dictionary(setting.aruco_intrinsic.dictionary).unwrap();
        let camera_matrix = Mat::from_slice_2d(&[
            &[setting.camera_intrinsic.fx, 0., setting.camera_intrinsic.cx],
            &[0., setting.camera_intrinsic.fy, setting.camera_intrinsic.cy],
            &[0., 0., 0.],
        ])
        .unwrap();
        let dist_coeffs = Vector::from_slice(setting.camera_distortion.as_slice());
        let mut detector_paramter = Ptr::new(DetectorParameters::default().unwrap());
        // detector_paramter.set_use_aruco3_detection(true);
        detector_paramter.set_adaptive_thresh_win_size_min(100);
        detector_paramter.set_adaptive_thresh_win_size_max(200);
        detector_paramter.set_adaptive_thresh_win_size_step(50);
        detector_paramter.set_min_marker_perimeter_rate(0.2);
        // detector_paramter.adaptiveThreshWinSizeMin = 100;
        // detector_paramter.adaptiveThreshWinSizeMax = 200;
        // detector_paramter.adaptiveThreshWinSizeStep = 50;
        // parameter.adaptiveThreshConstant = 7;
        // parameter.minMarkerPerimeterRate = 0.1;
        // parameter.maxMarkerPerimeterRate = 4;
        // parameter.polygonalApproxAccuracyRate = 0.1;
        // parameter.minCornerDistanceRate = 0.05;
        // parameter.minDistanceToBorder = 3;
        // parameter.minMarkerDistanceRate = 0.05;
        // // parameter.cornerRefinementMethod = cv.CORNER_REFINE_NONE;
        // // parameter.cornerRefinementWinSize = 5;
        // // parameter.cornerRefinementMaxIterations = 30;
        // // parameter.cornerRefinementMinAccuracy = 0.1;
        // parameter.markerBorderBits = 1;
        // parameter.perspectiveRemovePixelPerCell = 2;
        // parameter.perspectiveRemoveIgnoredMarginPerCell = 0.13;
        // parameter.maxErroneousBitsInBorderRate = 0.35;
        // parameter.minOtsuStdDev = 5.0;
        // parameter.errorCorrectionRate = 0.6;
        Self {
            dictionary,
            detector_paramter,
            setting,
            camera_matrix,
            dist_coeffs,
        }
    }

    pub fn find(
        &self,
        img: &impl ToInputArray,
        time_stamp: Duration,
        arucos: &mut Vec<Aruco>,
    ) -> Result<()> {
        arucos.clear();
        let mut corners = Vector::<Vector<Point2f>>::new();
        let mut ids = Vector::<i32>::new();
        let mut rvecs = Vector::<Vec3d>::new();
        let mut tvecs = Vector::<Vec3d>::new();
        // detect_markers_def(img, &self.dictionary, &mut corners, &mut ids)?;
        detect_markers(
            img,
            &self.dictionary,
            &mut corners,
            &mut ids,
            &self.detector_paramter,
            &mut no_array(),
        )?;
        if corners.is_empty() {
            return Ok(());
        }
        estimate_pose_single_markers_def(
            &corners,
            self.setting.aruco_intrinsic.marker_length,
            &self.camera_matrix,
            &self.dist_coeffs,
            &mut rvecs,
            &mut tvecs,
        )?;
        for (index, (id, (rvec, tvec))) in ids.iter().zip(rvecs.iter().zip(tvecs.iter())).enumerate() {
            let mut m = Mat::default();
            rodrigues_def(&rvec, &mut m)?;
            let m = nalgebra::Matrix3::from_iterator(m.iter::<f64>()?.map(|(_, v)| v));
            let (r, p, y) = Rotation3::from_matrix(&m).euler_angles();
            let c = corners.get(index).unwrap();
            let c = c.as_slice();
            arucos.push(Aruco {
                id,
                corners: [
                    [c[0].x, c[0].y],
                    [c[1].x, c[1].y],
                    [c[2].x, c[2].y],
                    [c[3].x, c[3].y],
                ],
                time_stamp,
                trans: [tvec.0[0], tvec.0[1], tvec.0[2]],
                euler_angles: [r, p, y],
            });
        }
        Ok(())
    }
}
