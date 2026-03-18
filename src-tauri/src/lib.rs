mod db;
mod models;
mod pdf;
mod queries;
mod reports;

use models::{DbConfig, GenerateParams, GenerateResult, PreviewData};

#[tauri::command]
async fn test_connection(config: DbConfig) -> Result<String, String> {
    let mut client = db::connect(&config).await?;
    let _result = client
        .simple_query("SELECT 1")
        .await
        .map_err(|e| format!("Query failed: {e}"))?;
    Ok("เชื่อมต่อฐานข้อมูลสำเร็จ".to_string())
}

#[tauri::command]
async fn preview_data(params: GenerateParams) -> Result<PreviewData, String> {
    let invoices =
        queries::fetch_invoices(&params.db_config, &params.start_date, &params.end_date).await?;
    let total_amount: f64 = invoices.iter().map(|i| i.total_cost).sum();
    let row_count = invoices.len();
    Ok(PreviewData {
        invoices,
        total_amount,
        row_count,
    })
}

#[tauri::command]
async fn generate_reports(params: GenerateParams) -> Result<GenerateResult, String> {
    let invoices =
        queries::fetch_invoices(&params.db_config, &params.start_date, &params.end_date).await?;

    if invoices.is_empty() {
        return Err("ไม่พบข้อมูลในช่วงวันที่ที่เลือก กรุณาตรวจสอบช่วงวันที่และฐานข้อมูล".to_string());
    }

    // Create output subfolder within the specified directory
    let output_base = if params.output_dir.trim().is_empty() {
        ".".to_string()
    } else {
        params.output_dir.trim().to_string()
    };

    // Create "output" subfolder
    let output_dir = std::path::Path::new(&output_base).join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| format!("ไม่สามารถสร้างโฟลเดอร์ output: {e}"))?;

    // Create params with updated output_dir for report generation
    let gen_params = GenerateParams {
        output_dir: output_dir.to_string_lossy().to_string(),
        ..params
    };

    let submission_rows = reports::process_invoice_submission(&invoices, &gen_params);
    let summary_rows = reports::process_receiving_summary(&invoices, &gen_params);
    let cover_pages = reports::process_cover_letters(&invoices, &gen_params);

    let invoice_file = pdf::generate_invoice_submission_pdf(&submission_rows, &gen_params)?;
    let summary_file = pdf::generate_receiving_summary_pdf(&summary_rows, &gen_params)?;
    let cover_files = pdf::generate_cover_letters_pdf(&cover_pages, &gen_params)?;

    let total_amount: f64 = invoices.iter().map(|i| i.total_cost).sum();

    let mut files = vec![invoice_file, summary_file];
    files.extend(cover_files);

    Ok(GenerateResult {
        files,
        total_rows: invoices.len(),
        total_amount,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            test_connection,
            preview_data,
            generate_reports
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
