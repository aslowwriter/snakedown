use std::path::PathBuf;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use color_eyre::Result;
use jupyter_protocol::Media;
use jupyter_protocol::MediaType;
use nbformat::v4::{Cell, Output};

use crate::render::formats::Renderer;

pub struct RenderedNotebook {
    pub text: String,
    pub images: Vec<DecodedDisplayData>,
}

#[derive(Clone)]
pub enum DecodedOutput {
    Text(String),
    Image(DecodedDisplayData),
}
#[derive(Debug, Clone)]
pub struct DecodedDisplayData {
    pub name: PathBuf,
    pub data: Vec<u8>,
}

pub fn rank_media_types(media: &MediaType) -> usize {
    match media {
        MediaType::Svg(_) => 10,
        MediaType::Png(_) => 9,
        MediaType::Jpeg(_) => 8,
        MediaType::Gif(_) => 7,
        MediaType::Html(_) => 6,
        MediaType::Json(_) => 5,
        MediaType::GeoJson(_) => 4,
        MediaType::Latex(_) => 3,
        MediaType::Markdown(_) => 2,
        MediaType::Plain(_) => 1,
        MediaType::DataTable(_)
        | MediaType::Javascript(_)
        | MediaType::Plotly(_)
        | MediaType::WidgetView(_)
        | MediaType::WidgetState(_)
        | MediaType::VegaLiteV2(_)
        | MediaType::VegaLiteV3(_)
        | MediaType::VegaLiteV4(_)
        | MediaType::VegaLiteV5(_)
        | MediaType::VegaLiteV6(_)
        | MediaType::VegaV3(_)
        | MediaType::VegaV4(_)
        | MediaType::VegaV5(_)
        | MediaType::Vdom(_)
        | MediaType::Other(_)
        | _ => 0,
    }
}

pub fn render_jupyter_display_data(cell_nr: usize, data: Media) -> Result<Option<DecodedOutput>> {
    let richest = data.richest(rank_media_types).map(|m| match m {
        MediaType::Svg(svg) => Ok(DecodedOutput::Image(DecodedDisplayData {
            name: PathBuf::from(cell_nr.to_string()).with_extension("svg"),
            data: svg.as_bytes().to_vec(),
        })),
        MediaType::Png(png) => {
            let decoded_data = BASE64_STANDARD.decode(png)?;
            Ok(DecodedOutput::Image(DecodedDisplayData {
                name: PathBuf::from(cell_nr.to_string()).with_extension("png"),
                data: decoded_data,
            }))
        }
        MediaType::Jpeg(jpg) => {
            let decoded_data = BASE64_STANDARD.decode(jpg)?;
            Ok(DecodedOutput::Image(DecodedDisplayData {
                name: PathBuf::from(cell_nr.to_string()).with_extension("jpg"),
                data: decoded_data,
            }))
        }
        MediaType::Gif(gif) => {
            let decoded_data = BASE64_STANDARD.decode(gif)?;
            Ok(DecodedOutput::Image(DecodedDisplayData {
                name: PathBuf::from(cell_nr.to_string()).with_extension("gif"),
                data: decoded_data,
            }))
        }
        MediaType::Html(html) => Ok(DecodedOutput::Text(html.clone())),
        MediaType::Json(json) => Ok(DecodedOutput::Text(json.clone().to_string())),
        MediaType::GeoJson(geojson) => Ok(DecodedOutput::Text(geojson.clone().to_string())),
        MediaType::Latex(latex) => Ok(DecodedOutput::Text(latex.clone())),
        MediaType::Markdown(md) => Ok(DecodedOutput::Text(md.clone())),
        MediaType::Plain(plain) => Ok(DecodedOutput::Text(plain.clone())),
        MediaType::DataTable(_)
        | MediaType::Javascript(_)
        | MediaType::Plotly(_)
        | MediaType::WidgetView(_)
        | MediaType::WidgetState(_)
        | MediaType::VegaLiteV2(_)
        | MediaType::VegaLiteV3(_)
        | MediaType::VegaLiteV4(_)
        | MediaType::VegaLiteV5(_)
        | MediaType::VegaLiteV6(_)
        | MediaType::VegaV3(_)
        | MediaType::VegaV4(_)
        | MediaType::VegaV5(_)
        | MediaType::Vdom(_)
        | MediaType::Other(_)
        | &_ => unreachable!(),
    });
    richest.transpose()
}

pub fn render_notebook<R: Renderer>(
    name: Option<&str>,
    notebook: &[Cell],
    renderer: &R,
) -> Result<RenderedNotebook> {
    let front_matter = renderer.render_front_matter(name);
    let mut rendered_cells = vec![front_matter];
    let mut rendered_notebook = RenderedNotebook {
        text: String::new(),
        images: Vec::new(),
    };
    for (count, cell) in notebook.iter().enumerate() {
        match cell {
            Cell::Markdown {
                id: _,
                metadata: _,
                source,
                attachments: _,
            } => rendered_cells.push(source.join("")),
            Cell::Code {
                id: _,
                metadata: _,
                execution_count: _,
                source,
                outputs,
            } => {
                rendered_cells.push(format!("```python\n{}\n```", source.join("")));
                for output in outputs {
                    match output {
                        Output::Stream { name: _, text } => {
                            rendered_cells.push(format!("```\n{}\n```", text.0.clone()));
                        }
                        Output::DisplayData(display_data) => {
                            if let Some(r) =
                                render_jupyter_display_data(count, display_data.data.clone())?
                            {
                                match r {
                                    DecodedOutput::Text(t) => rendered_cells.push(t),
                                    DecodedOutput::Image(img) => {
                                        rendered_cells
                                            .push(format!("![image]({})", img.name.display()));
                                        rendered_notebook.images.push(img);
                                    }
                                };
                            }
                        }
                        Output::ExecuteResult(exec_result) => {
                            if let Some(r) =
                                render_jupyter_display_data(count, exec_result.data.clone())?
                            {
                                match r {
                                    DecodedOutput::Text(t) => rendered_cells.push(t),
                                    DecodedOutput::Image(img) => {
                                        rendered_cells
                                            .push(format!("![image]({})", img.name.display()));
                                        rendered_notebook.images.push(img);
                                    }
                                };
                            }
                        }
                        Output::Error(_) => (),
                    };
                }
            }
            Cell::Raw {
                id: _,
                metadata: _,
                source,
            } => rendered_cells.push(format!("```\n{}\n```", source.join(""))),
        }
    }

    rendered_notebook.text = rendered_cells.join("\n\n");

    Ok(rendered_notebook)
}
#[cfg(test)]
mod test {}
