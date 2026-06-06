//! PDF generation for all three report types.
//!
//! Uses `printpdf 0.9` with an embedded Thai TrueType font (CordiaNew/THSarabun).
//! The font bytes are compiled into the binary via `include_bytes!`.
//!
//! # Output layout
//! - **ส่งหนี้เบิกยา** – one A4 landscape PDF (printpdf op-stream)
//! - **สรุปรับยา**     – one A4 landscape PDF (printpdf op-stream)
//! - **เบิกยาปะหน้า** – one A4 portrait PDF (lopdf + pre-built template overlay)

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod cover_letter;
mod cover_letter_template;
mod invoice_submission;
mod receiving_summary;
mod shared;

pub use cover_letter::generate_cover_letters_pdf;
pub use cover_letter_template::generate as generate_cover_letters_pdf_template;
pub use invoice_submission::generate_invoice_submission_pdf;
pub use receiving_summary::generate_receiving_summary_pdf;
