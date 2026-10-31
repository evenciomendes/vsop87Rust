use rust_astro_engine::*;
use std::env;
use std::time::Instant;

fn to_json(jd: f64, lng_observer: f64) -> String {
    let start = Instant::now();
    let bodies = calculate_all_bodies(jd);
    let true_obliq = nutation::get_true_obliquity_of_ecliptic(jd);
    let mean_obliq = nutation::get_mean_obliquity_of_ecliptic(jd);
    let nut_lon = nutation::get_nutation_in_longitude(jd);
    let nut_obliq = nutation::get_nutation_in_obliquity(jd);
    let lst = julianday::local_sidereal_time(jd, lng_observer);
    let elapsed_us = start.elapsed().as_micros();

    let mut bodies_json = String::from("[");
    for (i, b) in bodies.iter().enumerate() {
        if i > 0 {
            bodies_json.push(',');
        }
        let escaped_pos = b.formatted_position.replace('"', r#"\""#);
        bodies_json.push_str(&format!(
            r#"{{"id":"{}","name":"{}","symbol":"{}","category":"{}","longitude":{:.6},"latitude":{:.6},"rightAscension":{:.6},"declination":{:.6},"distance":{:.6},"distanceUnit":"{}","dailySpeed":{:.6},"isRetrograde":{},"signIndex":{},"signName":"{}","signSymbol":"{}","element":"{}","degreeInSign":{},"minutes":{},"seconds":{:.2},"formattedPosition":"{}"}}"#,
            b.id,
            b.name,
            b.symbol,
            b.category,
            b.longitude,
            b.latitude,
            b.right_ascension,
            b.declination,
            b.distance,
            b.distance_unit,
            b.daily_speed,
            b.is_retrograde,
            b.sign_index,
            b.sign_name,
            b.sign_symbol,
            b.element,
            b.degree_in_sign,
            b.minutes,
            b.seconds,
            escaped_pos
        ));
    }
    bodies_json.push(']');

    // Aspectos principais entre os 11 astros principais (Sol a Plutão + Quíron)
    let aspect_defs = [
        ("Conjunção", "☌", 0.0, 8.0, "#f59e0b"),
        ("Sextil", "⚹", 60.0, 5.0, "#10b981"),
        ("Quadratura", "□", 90.0, 7.0, "#ef4444"),
        ("Trígono", "△", 120.0, 7.0, "#3b82f6"),
        ("Oposição", "☍", 180.0, 8.0, "#ec4899"),
    ];
    let mut aspects_json = String::from("[");
    let mut first_asp = true;
    let limit = bodies.len().min(11);
    for i in 0..limit {
        for j in (i + 1)..limit {
            let mut diff = (bodies[i].longitude - bodies[j].longitude).abs();
            if diff > 180.0 {
                diff = 360.0 - diff;
            }
            for &(aname, asym, exact_angle, max_orb, color) in &aspect_defs {
                let orb = (diff - exact_angle).abs();
                if orb <= max_orb {
                    if !first_asp {
                        aspects_json.push(',');
                    }
                    first_asp = false;
                    aspects_json.push_str(&format!(
                        r#"{{"body1":"{}","symbol1":"{}","body2":"{}","symbol2":"{}","aspectName":"{}","aspectSymbol":"{}","angle":{:.2},"orb":{:.2},"color":"{}"}}"#,
                        bodies[i].name,
                        bodies[i].symbol,
                        bodies[j].name,
                        bodies[j].symbol,
                        aname,
                        asym,
                        diff,
                        orb,
                        color
                    ));
                }
            }
        }
    }
    aspects_json.push(']');

    format!(
        r#"{{"engine":"rust-astro-engine v1.0 (VSOP87 + ELP2000 + Meeus)","julianDay":{:.6},"unixMs":{:.0},"computationTimeMicros":{},"astronomy":{{"trueObliquity":{:.6},"meanObliquity":{:.6},"nutationLongitudeArcsec":{:.4},"nutationObliquityArcsec":{:.4},"localSiderealTimeHours":{:.6}}},"bodies":{},"aspects":{}}}"#,
        jd,
        julianday::to_unix_ms(jd),
        elapsed_us,
        true_obliq,
        mean_obliq,
        nut_lon,
        nut_obliq,
        lst,
        bodies_json,
        aspects_json
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut jd = julianday::from_utc(2026, 10, 30, 17, 0, 0.0);
    let mut lng = -46.6333; // São Paulo default

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--jd" if i + 1 < args.len() => {
                jd = args[i + 1].parse().unwrap_or(jd);
                i += 2;
            }
            "--unix-ms" if i + 1 < args.len() => {
                if let Ok(ms) = args[i + 1].parse::<f64>() {
                    jd = julianday::from_unix_ms(ms);
                }
                i += 2;
            }
            "--lng" if i + 1 < args.len() => {
                lng = args[i + 1].parse().unwrap_or(lng);
                i += 2;
            }
            "--utc" if i + 6 < args.len() => {
                let y = args[i + 1].parse().unwrap_or(2026);
                let m = args[i + 2].parse().unwrap_or(1);
                let d = args[i + 3].parse().unwrap_or(1);
                let h = args[i + 4].parse().unwrap_or(0);
                let min = args[i + 5].parse().unwrap_or(0);
                let sec = args[i + 6].parse().unwrap_or(0.0);
                jd = julianday::from_utc(y, m, d, h, min, sec);
                i += 7;
            }
            _ => {
                i += 1;
            }
        }
    }

    println!("{}", to_json(jd, lng));
}
