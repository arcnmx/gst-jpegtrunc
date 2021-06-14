mod jfif;

use glib::subclass::{self, prelude::*};
use gstreamer::{Plugin, Element, Caps, PadTemplate, PadPresence, PadDirection, Buffer, BufferRef, Rank, BUFFER_COPY_ALL, FlowSuccess, FlowError, gst_error};
use gstreamer::subclass::prelude::*;
use gstreamer_base::BaseTransform;
use gstreamer_base::subclass::prelude::*;
use gstreamer_base::subclass::base_transform::PrepareOutputBufferSuccess;
use once_cell::sync::Lazy;

#[derive(Default)]
pub struct JpegTrunc {
}

fn plugin_init(plugin: &Plugin) -> Result<(), glib::BoolError> {
	Element::register(Some(plugin),
		"jpegtrunc", Rank::None, JpegTrunc::get_type())?;

	Ok(())
}

gstreamer::gst_plugin_define! {
	jpegtrunc,
	env!("CARGO_PKG_DESCRIPTION"),
	plugin_init,
	env!("CARGO_PKG_VERSION"),
	"MIT/X11",
	env!("CARGO_PKG_NAME"),
	env!("CARGO_PKG_NAME"),
	env!("CARGO_PKG_REPOSITORY"),
	"2000-01-01"
}

impl ObjectSubclass for JpegTrunc {
	const NAME: &'static str = "GstJpegTrunc";
	type ParentType = BaseTransform;
	type Instance = gstreamer::subclass::ElementInstanceStruct<Self>;
	type Class = subclass::simple::ClassStruct<Self>;

	glib::glib_object_subclass!();

	fn class_init(klass: &mut Self::Class) {
		klass.set_metadata(
			"JPEG Truncate",
			"Filters/Whatever",
			env!("CARGO_PKG_DESCRIPTION"),
			env!("CARGO_PKG_AUTHORS"),
		);

		let caps = Caps::new_simple(
			"image/jpeg",
			&[],
		);

		klass.add_pad_template(PadTemplate::new(
			"src",
			PadDirection::Src,
			PadPresence::Always,
			&caps,
		).unwrap());

		klass.add_pad_template(PadTemplate::new(
			"sink",
			PadDirection::Sink,
			PadPresence::Always,
			&caps,
		).unwrap());

		klass.install_properties(&PROPERTIES);

		klass.configure(
			gstreamer_base::subclass::BaseTransformMode::AlwaysInPlace,
			true,
			true,
		);
	}

	fn new() -> Self {
		Self::default()
	}
}

static PROPERTIES: [subclass::Property; 0] = [ ];
static CAT: Lazy<gstreamer::DebugCategory> = Lazy::new(|| {
	gstreamer::DebugCategory::new(
		"jpegtrunc",
		gstreamer::DebugColorFlags::empty(),
		Some("JPEG Truncate"),
	)
});

impl ObjectImpl for JpegTrunc {
	glib::glib_object_impl!();

	/*fn set_property(&self, _obj: &Object, _id: usize, _value: &Value) {
		todo!()
	}

	fn get_property(&self, _obj: &Object, _id: usize) -> Result<Value, ()> {
		todo!()
	}*/
}

impl ElementImpl for JpegTrunc {

}

fn find_eoi(element: &BaseTransform, buffer: &[u8], ignore_garbage: bool) -> Option<usize> {
	use jfif::{markers, marker_has_length};

	let mut contents = match buffer {
		&[m0, m1, ref rest @ .. ] if m0 == markers::P && m1 == markers::SOI => rest,
		_ => {
			gst_error!(CAT,
				obj: element,
				"header not found");
			return None
		},
	};

	'outer: loop {
		let (marker, rest) = match contents {
			&[m0, m1, ref rest @ .. ] if m0 == markers::P => (m1, rest),
			&[_m, ref rest @ .. ] if ignore_garbage => {
				contents = rest;
				continue
			},
			_ => {
				gst_error!(CAT,
					obj: element,
					"unexpected EOF");
				return None
			},
		};
		contents = rest;

		match marker {
			marker if marker_has_length(marker) => {
				let segment_len = match contents {
					&[l0, l1, ref rest @ .. ] => {
						contents = rest;
						u16::from_be_bytes([l0, l1])
					},
					_ => {
						contents = &[]; // trigger eof
						continue
					},
				};
				let segment = segment_len.checked_sub(2)
					.and_then(|segment_len| contents.get(segment_len as usize..));

				// skip the whole thing...
				contents = match segment {
					Some(rest) => rest,
					None => {
						gst_error!(CAT,
							obj: element,
							"invalid segment length");
						return None
					},
				};

				if marker == markers::SOS {
					// entropy data following the SOS segment (just a header)
					while let Some(found) = memchr::memchr(markers::P, contents) {
						let rest = unsafe {
							// we assume memchr knows what it's doing
							contents.get_unchecked((found + 1)..)
						};
						match rest {
							&[marker, ref rest @ .. ] => {
								contents = rest;
								if marker == markers::EOI {
									break 'outer
								}
							},
							rest @ &[] => {
								contents = rest;
							},
						}
					}
				}
			},
			markers::EOI => break,
			_marker => {
				()
			},
		}
	}

	Some(buffer.len() - contents.len())
}

impl BaseTransformImpl for JpegTrunc {
	fn prepare_output_buffer(&self, element: &BaseTransform, inbuf: &BufferRef) -> Result<PrepareOutputBufferSuccess, FlowError> {
		let new_len = {
			let buffer = inbuf.map_readable()
				.map_err(|e| {
					gst_error!(CAT,
						obj: element,
						"Failed to read buffer memory: {:?}", e);
					FlowError::NotSupported
				})?;
			find_eoi(element, &buffer, false)
		};

		Ok(match new_len {
			None => PrepareOutputBufferSuccess::InputBuffer,
			Some(new_len) => {
				PrepareOutputBufferSuccess::Buffer(inbuf.copy_region(
					BUFFER_COPY_ALL,
					0, Some(new_len),
				).map_err(|e| {
					gst_error!(CAT,
						obj: element,
						"Failed to create output buffer: {:?}", e);
					FlowError::Error
				})?)
			},
		})
	}

	fn transform_ip_passthrough(&self, _element: &BaseTransform, _buf: &Buffer) -> Result<FlowSuccess, FlowError> {
		Ok(FlowSuccess::Ok)
	}
}
