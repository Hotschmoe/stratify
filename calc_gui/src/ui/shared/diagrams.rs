//! Canvas drawing utilities for beam diagrams
//!
//! Renders beam schematics with supports, shear diagrams, moment diagrams,
//! and deflection diagrams.

use iced::widget::canvas::{self, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Theme};

use calc_core::calculations::continuous_beam::{ContinuousBeamInput, ContinuousBeamResult, SupportType};

use crate::Message;

/// Data needed to draw beam diagrams
pub struct BeamDiagramData {
    pub total_length_ft: f64,
    #[allow(dead_code)]
    pub load_plf: f64,
    pub max_shear_lb: f64,
    pub max_moment_ftlb: f64,
    pub max_deflection_in: f64,
    // Multi-span support
    pub span_lengths_ft: Vec<f64>,
    pub support_types: Vec<SupportType>,
    pub reactions: Vec<f64>,
    // Pre-computed diagram points from analysis
    pub shear_diagram: Vec<(f64, f64)>,
    pub moment_diagram: Vec<(f64, f64)>,
    pub deflection_diagram: Vec<(f64, f64)>,
}

impl BeamDiagramData {
    pub fn from_calc(input: &ContinuousBeamInput, result: &ContinuousBeamResult) -> Self {
        Self {
            total_length_ft: input.total_length_ft(),
            load_plf: input.load_case.total_uniform_plf(),
            max_shear_lb: result.max_shear_lb,
            max_moment_ftlb: result.max_positive_moment_ftlb,
            max_deflection_in: result.max_deflection_in,
            span_lengths_ft: input.spans.iter().map(|s| s.length_ft).collect(),
            support_types: input.supports.clone(),
            reactions: result.reactions.clone(),
            shear_diagram: result.shear_diagram.clone(),
            moment_diagram: result.moment_diagram.clone(),
            deflection_diagram: result.deflection_diagram.clone(),
        }
    }

    /// Check if this is a multi-span beam
    #[allow(dead_code)]
    pub fn is_multi_span(&self) -> bool {
        self.span_lengths_ft.len() > 1
    }

    /// Get node positions (cumulative span lengths starting from 0)
    pub fn node_positions_ft(&self) -> Vec<f64> {
        let mut positions = vec![0.0];
        let mut cumulative = 0.0;
        for len in &self.span_lengths_ft {
            cumulative += len;
            positions.push(cumulative);
        }
        positions
    }
}

/// Canvas program for drawing beam diagrams
pub struct BeamDiagram {
    data: BeamDiagramData,
}

impl BeamDiagram {
    pub fn new(data: BeamDiagramData) -> Self {
        Self { data }
    }

    fn draw_beam_schematic(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        let beam_y = y + height * 0.55;
        let beam_thickness = 4.0;
        let support_size = 10.0;
        let total_length = self.data.total_length_ft;
        let reaction_color = Color::from_rgb(0.7, 0.2, 0.2);

        // Draw beam line
        let beam = Path::line(
            Point::new(x, beam_y),
            Point::new(x + width, beam_y),
        );
        frame.stroke(&beam, Stroke::default().with_color(color).with_width(beam_thickness));

        // Get node positions
        let node_positions = self.data.node_positions_ft();

        // Draw supports at each node
        for (i, &node_ft) in node_positions.iter().enumerate() {
            let node_x = x + (node_ft / total_length) as f32 * width;
            let support_type = self.data.support_types.get(i).copied().unwrap_or(SupportType::Pinned);

            self.draw_support(frame, node_x, beam_y + beam_thickness / 2.0, support_size, support_type, color);

            // Draw reaction arrow and label for supported nodes (not Free)
            if support_type.restrains_vertical() {
                let reaction = self.data.reactions.get(i).copied().unwrap_or(0.0);
                if reaction.abs() > 1.0 {
                    let reaction_arrow_length = height * 0.15;
                    let reaction_start_y = beam_y + support_size + 8.0;

                    // Draw reaction arrow (upward for positive, downward for negative)
                    let (arrow_start, arrow_end) = if reaction >= 0.0 {
                        (reaction_start_y + reaction_arrow_length, reaction_start_y)
                    } else {
                        (reaction_start_y, reaction_start_y + reaction_arrow_length)
                    };

                    let reaction_arrow = Path::line(
                        Point::new(node_x, arrow_start),
                        Point::new(node_x, arrow_end),
                    );
                    frame.stroke(
                        &reaction_arrow,
                        Stroke::default().with_color(reaction_color).with_width(2.0),
                    );

                    // Arrow head
                    let head_y = arrow_end;
                    let head_dir = if reaction >= 0.0 { 1.0 } else { -1.0 };
                    let head = Path::new(|builder| {
                        builder.move_to(Point::new(node_x, head_y));
                        builder.line_to(Point::new(node_x - 3.0, head_y + head_dir * 6.0));
                        builder.move_to(Point::new(node_x, head_y));
                        builder.line_to(Point::new(node_x + 3.0, head_y + head_dir * 6.0));
                    });
                    frame.stroke(
                        &head,
                        Stroke::default().with_color(reaction_color).with_width(2.0),
                    );

                    // Reaction label - show R_1, R_2, etc.
                    let label = format!("R_{} = {:.0}", i + 1, reaction.abs());
                    let label_x = if i == 0 { node_x + 3.0 } else { node_x - 45.0 };
                    let reaction_text = Text {
                        content: label,
                        position: Point::new(label_x, reaction_start_y + reaction_arrow_length + 2.0),
                        color: reaction_color,
                        size: iced::Pixels(8.0),
                        ..Text::default()
                    };
                    frame.fill_text(reaction_text);
                }
            }
        }

        // Draw uniform load arrows
        let num_arrows = 8.min((total_length * 2.0) as i32).max(4);
        let arrow_spacing = width / (num_arrows as f32);
        let arrow_length = height * 0.2;

        for i in 0..=num_arrows {
            let ax = x + i as f32 * arrow_spacing;
            let arrow = Path::line(
                Point::new(ax, beam_y - arrow_length),
                Point::new(ax, beam_y - 5.0),
            );
            frame.stroke(&arrow, Stroke::default().with_color(color).with_width(1.0));

            // Arrow head
            let head = Path::new(|builder| {
                builder.move_to(Point::new(ax, beam_y - 5.0));
                builder.line_to(Point::new(ax - 2.0, beam_y - 9.0));
                builder.move_to(Point::new(ax, beam_y - 5.0));
                builder.line_to(Point::new(ax + 2.0, beam_y - 9.0));
            });
            frame.stroke(&head, Stroke::default().with_color(color).with_width(1.0));
        }

        // Load label
        let load_text = Text {
            content: format!("w = {:.0} plf", self.data.load_plf),
            position: Point::new(x + width / 2.0, y + 5.0),
            color,
            size: iced::Pixels(9.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..Text::default()
        };
        frame.fill_text(load_text);

        // Span labels - one for each span
        for (i, span_len) in self.data.span_lengths_ft.iter().enumerate() {
            let start = node_positions[i];
            let end = node_positions[i + 1];
            let mid = (start + end) / 2.0;
            let span_x = x + (mid / total_length) as f32 * width;

            let span_text = Text {
                content: format!("L{} = {:.1}'", i + 1, span_len),
                position: Point::new(span_x, beam_y + support_size + 5.0),
                color,
                size: iced::Pixels(8.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                ..Text::default()
            };
            frame.fill_text(span_text);
        }
    }

    /// Draw a support symbol at the given position
    fn draw_support(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        size: f32,
        support_type: SupportType,
        color: Color,
    ) {
        match support_type {
            SupportType::Pinned => {
                // Triangle (filled)
                let support = Path::new(|builder| {
                    builder.move_to(Point::new(x, y));
                    builder.line_to(Point::new(x - size / 2.0, y + size));
                    builder.line_to(Point::new(x + size / 2.0, y + size));
                    builder.close();
                });
                frame.fill(&support, color);
            }
            SupportType::Roller => {
                // Triangle with circle underneath
                let triangle = Path::new(|builder| {
                    builder.move_to(Point::new(x, y));
                    builder.line_to(Point::new(x - size / 2.0, y + size * 0.7));
                    builder.line_to(Point::new(x + size / 2.0, y + size * 0.7));
                    builder.close();
                });
                frame.stroke(&triangle, Stroke::default().with_color(color).with_width(2.0));

                // Circle
                let circle_radius = size * 0.15;
                let circle = Path::circle(Point::new(x, y + size * 0.7 + circle_radius + 1.0), circle_radius);
                frame.stroke(&circle, Stroke::default().with_color(color).with_width(2.0));
            }
            SupportType::Fixed => {
                // Filled rectangle with hatching
                let rect_height = size;
                let rect_width = size * 0.3;

                let rect = Path::new(|builder| {
                    builder.move_to(Point::new(x - rect_width, y));
                    builder.line_to(Point::new(x + rect_width, y));
                    builder.line_to(Point::new(x + rect_width, y + rect_height));
                    builder.line_to(Point::new(x - rect_width, y + rect_height));
                    builder.close();
                });
                frame.fill(&rect, color);

                // Hatching lines
                for i in 0..3 {
                    let hatch_y = y + (i as f32 + 0.5) * rect_height / 3.0;
                    let hatch = Path::line(
                        Point::new(x - rect_width - 3.0, hatch_y + 3.0),
                        Point::new(x + rect_width + 3.0, hatch_y - 3.0),
                    );
                    frame.stroke(&hatch, Stroke::default().with_color(color).with_width(1.0));
                }
            }
            SupportType::Free => {
                // No support symbol - maybe a small dot to show the end
                let dot = Path::circle(Point::new(x, y + 2.0), 2.0);
                frame.stroke(&dot, Stroke::default().with_color(color).with_width(1.5));
            }
        }
    }

    fn draw_shear_diagram(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        axis_color: Color,
    ) {
        let center_y = y + height / 2.0;
        let plot_height = height * 0.35;

        // Axis line
        let axis = Path::line(
            Point::new(x, center_y),
            Point::new(x + width, center_y),
        );
        frame.stroke(&axis, Stroke::default().with_color(axis_color).with_width(1.0));

        // Draw shear diagram using pre-computed points
        if !self.data.shear_diagram.is_empty() && self.data.max_shear_lb.abs() > 1e-6 {
            // Find min/max shear for scaling
            let max_v = self.data.shear_diagram.iter()
                .map(|(_, v)| v.abs())
                .fold(0.0f64, |a, b| a.max(b));

            if max_v > 1e-6 {
                // Draw filled area
                let shear_path = Path::new(|builder| {
                    let first = &self.data.shear_diagram[0];
                    let px = x + (first.0 as f32 / self.data.total_length_ft as f32) * width;
                    let v_norm = first.1 / max_v;
                    let py = center_y - (v_norm as f32) * plot_height;
                    builder.move_to(Point::new(px, center_y));
                    builder.line_to(Point::new(px, py));

                    for (pos, v) in &self.data.shear_diagram {
                        let px = x + (*pos as f32 / self.data.total_length_ft as f32) * width;
                        let v_norm = v / max_v;
                        let py = center_y - (v_norm as f32) * plot_height;
                        builder.line_to(Point::new(px, py));
                    }

                    let last = self.data.shear_diagram.last().unwrap();
                    let px = x + (last.0 as f32 / self.data.total_length_ft as f32) * width;
                    builder.line_to(Point::new(px, center_y));
                    builder.close();
                });
                frame.fill(&shear_path, Color { a: 0.3, ..color });

                // Draw line
                let shear_line = Path::new(|builder| {
                    let first = &self.data.shear_diagram[0];
                    let px = x + (first.0 as f32 / self.data.total_length_ft as f32) * width;
                    let v_norm = first.1 / max_v;
                    let py = center_y - (v_norm as f32) * plot_height;
                    builder.move_to(Point::new(px, py));

                    for (pos, v) in &self.data.shear_diagram {
                        let px = x + (*pos as f32 / self.data.total_length_ft as f32) * width;
                        let v_norm = v / max_v;
                        let py = center_y - (v_norm as f32) * plot_height;
                        builder.line_to(Point::new(px, py));
                    }
                });
                frame.stroke(&shear_line, Stroke::default().with_color(color).with_width(2.0));
            }
        }

        // Labels
        let title = Text {
            content: "Shear (V)".to_string(),
            position: Point::new(x + 5.0, y + 5.0),
            color,
            size: iced::Pixels(10.0),
            ..Text::default()
        };
        frame.fill_text(title);

        let max_label = Text {
            content: format!("+{:.0} lb", self.data.max_shear_lb),
            position: Point::new(x + 5.0, center_y - plot_height - 2.0),
            color,
            size: iced::Pixels(9.0),
            ..Text::default()
        };
        frame.fill_text(max_label);

        let min_label = Text {
            content: format!("-{:.0} lb", self.data.max_shear_lb),
            position: Point::new(x + width - 50.0, center_y + plot_height + 10.0),
            color,
            size: iced::Pixels(9.0),
            ..Text::default()
        };
        frame.fill_text(min_label);
    }

    fn draw_moment_diagram(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        axis_color: Color,
    ) {
        let axis_y = y + height * 0.15;
        let plot_height = height * 0.7;

        // Axis line
        let axis = Path::line(
            Point::new(x, axis_y),
            Point::new(x + width, axis_y),
        );
        frame.stroke(&axis, Stroke::default().with_color(axis_color).with_width(1.0));

        // Draw moment diagram using pre-computed points
        if !self.data.moment_diagram.is_empty() && self.data.max_moment_ftlb.abs() > 1e-6 {
            let max_m = self.data.max_moment_ftlb;

            // Draw filled area
            let moment_path = Path::new(|builder| {
                builder.move_to(Point::new(x, axis_y));
                for (pos, m) in &self.data.moment_diagram {
                    let px = x + (*pos as f32 / self.data.total_length_ft as f32) * width;
                    let m_ratio = m / max_m;
                    let py = axis_y + (m_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
                builder.line_to(Point::new(x + width, axis_y));
                builder.close();
            });
            frame.fill(&moment_path, Color { a: 0.3, ..color });

            // Draw outline
            let outline = Path::new(|builder| {
                let first = &self.data.moment_diagram[0];
                let px = x + (first.0 as f32 / self.data.total_length_ft as f32) * width;
                let m_ratio = first.1 / max_m;
                let py = axis_y + (m_ratio as f32) * plot_height;
                builder.move_to(Point::new(px, py));

                for (pos, m) in &self.data.moment_diagram {
                    let px = x + (*pos as f32 / self.data.total_length_ft as f32) * width;
                    let m_ratio = m / max_m;
                    let py = axis_y + (m_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
            });
            frame.stroke(&outline, Stroke::default().with_color(color).with_width(2.0));
        }

        // Labels
        let title = Text {
            content: "Moment (M)".to_string(),
            position: Point::new(x + 5.0, y + 5.0),
            color,
            size: iced::Pixels(10.0),
            ..Text::default()
        };
        frame.fill_text(title);

        let max_label = Text {
            content: format!("{:.0} ft-lb", self.data.max_moment_ftlb),
            position: Point::new(x + width / 2.0, axis_y + plot_height + 10.0),
            color,
            size: iced::Pixels(9.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..Text::default()
        };
        frame.fill_text(max_label);
    }

    fn draw_deflection_diagram(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        axis_color: Color,
    ) {
        let axis_y = y + height * 0.15;
        let plot_height = height * 0.6;

        // Axis line (represents undeflected beam)
        let axis = Path::line(
            Point::new(x, axis_y),
            Point::new(x + width, axis_y),
        );
        frame.stroke(&axis, Stroke::default().with_color(axis_color).with_width(1.0));

        // Draw deflection using pre-computed points
        if !self.data.deflection_diagram.is_empty() && self.data.max_deflection_in.abs() > 1e-9 {
            let max_d = self.data.max_deflection_in;

            // Draw curve
            let defl_path = Path::new(|builder| {
                let first = &self.data.deflection_diagram[0];
                let px = x + (first.0 as f32 / self.data.total_length_ft as f32) * width;
                let d_ratio = first.1 / max_d;
                let py = axis_y + (d_ratio as f32) * plot_height;
                builder.move_to(Point::new(px, py));

                for (pos, d) in &self.data.deflection_diagram {
                    let px = x + (*pos as f32 / self.data.total_length_ft as f32) * width;
                    let d_ratio = d / max_d;
                    let py = axis_y + (d_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
            });
            frame.stroke(&defl_path, Stroke::default().with_color(color).with_width(2.0));

            // Fill under curve
            let fill_path = Path::new(|builder| {
                builder.move_to(Point::new(x, axis_y));
                for (pos, d) in &self.data.deflection_diagram {
                    let px = x + (*pos as f32 / self.data.total_length_ft as f32) * width;
                    let d_ratio = d / max_d;
                    let py = axis_y + (d_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
                builder.line_to(Point::new(x + width, axis_y));
                builder.close();
            });
            frame.fill(&fill_path, Color { a: 0.2, ..color });
        }

        // Labels
        let title = Text {
            content: "Deflection (δ)".to_string(),
            position: Point::new(x + 5.0, y + 5.0),
            color,
            size: iced::Pixels(10.0),
            ..Text::default()
        };
        frame.fill_text(title);

        let max_label = Text {
            content: format!("{:.3} in", self.data.max_deflection_in),
            position: Point::new(x + width / 2.0, axis_y + plot_height + 10.0),
            color,
            size: iced::Pixels(9.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            ..Text::default()
        };
        frame.fill_text(max_label);
    }
}

impl canvas::Program<Message> for BeamDiagram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let width = bounds.width;
        let height = bounds.height;

        // Layout: divide into 4 sections vertically
        let section_height = height / 4.0;
        let margin = 20.0;
        let plot_width = width - 2.0 * margin;

        // Colors
        let beam_color = Color::from_rgb(0.3, 0.3, 0.3);
        let shear_color = Color::from_rgb(0.2, 0.5, 0.8);
        let moment_color = Color::from_rgb(0.8, 0.4, 0.2);
        let defl_color = Color::from_rgb(0.2, 0.7, 0.3);
        let axis_color = Color::from_rgb(0.5, 0.5, 0.5);

        // Section 1: Beam schematic with uniform load
        self.draw_beam_schematic(&mut frame, margin, 0.0, plot_width, section_height, beam_color);

        // Section 2: Shear diagram
        self.draw_shear_diagram(
            &mut frame,
            margin,
            section_height,
            plot_width,
            section_height,
            shear_color,
            axis_color,
        );

        // Section 3: Moment diagram
        self.draw_moment_diagram(
            &mut frame,
            margin,
            section_height * 2.0,
            plot_width,
            section_height,
            moment_color,
            axis_color,
        );

        // Section 4: Deflection diagram
        self.draw_deflection_diagram(
            &mut frame,
            margin,
            section_height * 3.0,
            plot_width,
            section_height,
            defl_color,
            axis_color,
        );

        vec![frame.into_geometry()]
    }
}
