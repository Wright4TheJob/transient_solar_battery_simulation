use chrono::{Datelike, NaiveDateTime};
use plotters::coord::types::RangedDateTime;
use plotters::prelude::*;
pub fn chart(
    xs: &[NaiveDateTime],
    ys: &[Vec<f32>],
    ys_secondary: &[Vec<f32>],
    labels: &[String],
    title: Option<&String>,
    show_legend: bool,
) {
    let output_file = "Energy Plot.png";

    let root = BitMapBackend::new(output_file, (1024, 768)).into_drawing_area();
    let mut builder = ChartBuilder::on(&root);
    //use plotters::{prelude::*, style::Color};
    root.fill(&WHITE).unwrap();

    //const PLOT_LINE_COLOR: RGBColor = RGBColor(0, 175, 255);

    let from_date = *xs.first().expect("No dates to display");
    let to_date = *xs.last().expect("No dates to display");

    let y_max: f32 = ys
        .iter()
        .map(|y| y.clone().into_iter().reduce(f32::max))
        .filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .reduce(f32::max)
        .unwrap();

    let y_secondary_max: f32 = ys_secondary
        .iter()
        .map(|y| y.clone().into_iter().reduce(f32::max))
        .filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .reduce(f32::max)
        .unwrap();

    let mut chart = if title.is_some() {
        builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .right_y_label_area_size(40)
            .margin(20_i32)
            .caption(title.clone().unwrap().as_str(), ("sans-serif", 30.0))
            .build_cartesian_2d(
                RangedDateTime::from(from_date..to_date),
                0_f32..y_max * 1.05,
            )
            .unwrap()
            .set_secondary_coord(
                RangedDateTime::from(from_date..to_date),
                0_f32..y_secondary_max * 1.05,
            )
    } else {
        builder
            .x_label_area_size(28_i32)
            .y_label_area_size(28_i32)
            .right_y_label_area_size(40)
            .margin(20_i32)
            .build_cartesian_2d(
                RangedDateTime::from(from_date..to_date),
                0_f32..y_max * 1.05,
            )
            .unwrap()
            .set_secondary_coord(
                RangedDateTime::from(from_date..to_date),
                0_f32..y_secondary_max * 1.05,
            )
        // .expect("Failed to build chart")
    };

    chart
        .configure_mesh()
        //.bold_line_style(plotters::style::colors::BLUE.mix(0.1))
        //.light_line_style(plotters::style::colors::BLUE.mix(0.05))
        //.axis_style(ShapeStyle::from(plotters::style::colors::BLUE.mix(0.45)).stroke_width(1))
        //.y_labels(10)
        .x_labels(6)
        .x_label_formatter(&|x| format!("{}-{}-{}", x.day(), x.month(), x.year()))
        //.y_label_style(
        //    ("sans-serif", 15)
        //        .into_font()
        //        .color(&plotters::style::colors::BLUE.mix(0.65))
        //        .transform(FontTransform::Rotate90),
        //)
        .y_label_formatter(&|y| format!("{y}"))
        .y_desc("Battery Charge")
        .draw()
        .expect("failed to draw chart mesh");

    chart
        .configure_secondary_axes()
        .y_desc("Daylight Hours")
        .draw()
        .unwrap();

    let colors = [
        &BLUE,
        &RED,
        &BLACK,
        &RGBColor(0, 128, 0),     // green
        &RGBColor(255, 146, 0),   // Orange/brown
        &RGBColor(0, 153, 230),   // light blue
        &RGBColor(180, 0, 180),   // Purple
        &RGBColor(255, 150, 150), // pink
    ];
    let mut color_index = 0;
    let n = *[ys.len(), colors.len(), labels.len()]
        .iter()
        .min()
        .unwrap_or(&1);

    for i in 0..n {
        let this_data: Vec<(NaiveDateTime, f32)> = xs.iter().copied().zip(ys[i].clone()).collect();
        let this_color = colors[color_index];
        let this_label = labels[i].clone();
        chart
            .draw_series(
                LineSeries::new(
                    this_data.iter().copied(),
                    this_color,
                    //PLOT_LINE_COLOR.mix(0.175),
                ), //.border_style(ShapeStyle::from(**color).stroke_width(2)),
            )
            .expect("failed to draw chart data")
            .label(this_label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], *this_color));
        color_index += 1;
    }

    let n = *[ys_secondary.len(), colors.len(), labels.len()]
        .iter()
        .min()
        .unwrap_or(&1);

    for i in 0..n {
        let this_data: Vec<(NaiveDateTime, f32)> = xs
            .iter()
            .copied()
            .zip(ys_secondary.get(i).unwrap().clone())
            .collect();
        let this_color = *colors.get(color_index).unwrap();
        let this_label = labels.get(color_index).unwrap();
        chart
            .draw_secondary_series(
                LineSeries::new(
                    this_data.iter().copied(),
                    this_color,
                    //PLOT_LINE_COLOR.mix(0.175),
                ), //.border_style(ShapeStyle::from(**color).stroke_width(2)),
            )
            .expect("failed to draw chart data")
            .label(this_label)
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], *this_color));
        color_index += 1;
    }

    if show_legend {
        chart
            .configure_series_labels()
            .background_style(WHITE)
            .border_style(BLACK)
            .draw()
            .expect("Failed to draw legend")
    }
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {output_file}");
}
