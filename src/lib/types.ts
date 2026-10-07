export interface DbConfig {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
}

export interface InvoiceRow {
    invoice_no: string;
    vendor_code: string;
    company_name: string;
    company_keyword: string;
    total_cost: number;
    receive_date: string;
    category: string;
}

export interface PreviewData {
    invoices: InvoiceRow[];
    total_amount: number;
    row_count: number;
}

export interface CarryForward {
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    next_purchase_no: number;
    remaining_balance: number;
}

export interface GenerateResult {
    files: string[];
    total_rows: number;
    total_amount: number;
    carry_forward: CarryForward;
}

export interface SkippedLockedNumberSet {
    request_no: number;
    report_no: number;
    purchase_no: number;
    reason: string;
    note: string;
}

export interface ReceivingNumberingInfo {
    start_po_no: number;
    start_purchase_no: number;
    skipped_locked_sets: SkippedLockedNumberSet[];
}

export interface RoundHistoryEntry {
    id: string;
    label: string;
    fiscal_year: number;
    month: number;
    round: number;
    date_from: string;
    date_to: string;
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    next_purchase_no?: number;
    remaining_balance: number;
    budget_total: number;
    total_amount: number;
    invoice_count: number;
    source_tab?: string;
    created_at: string;
}

export interface NumberLockEntry {
    id: string;
    fiscal_year: number;
    request_no: number;
    report_no: number;
    purchase_no: number;
    reason: string;
    note: string;
    created_at: string;
}

export interface InvoiceSubmissionRow {
    seq: number;
    receive_date: string;
    invoice_no: string;
    reg_no: string;
    running_in_reg: number;
    invoice_date: string;
    company_name: string;
    category: string;
    total_amount: number;
}

export interface InvoiceSubmissionPreview {
    rows: InvoiceSubmissionRow[];
    carry_forward: CarryForward;
    total_rows: number;
    total_amount: number;
}

export interface ReceivingSummaryRow {
    approval_date: string;
    po_date: string;
    receive_date: string;
    company_code: string;
    total_amount: number;
    receiving_code: number;
    reg_no: string;
    running_in_reg: number;
    invoice_no: string;
    request_no: number;
    report_no: number;
    po_no: number;
}

export interface ReceivingSummaryPreview {
    rows: ReceivingSummaryRow[];
    carry_forward: CarryForward;
    total_rows: number;
    total_amount: number;
    numbering_info: ReceivingNumberingInfo;
}

export interface ReceivingSummaryGenerateResult {
    files: string[];
    total_rows: number;
    total_amount: number;
    carry_forward: CarryForward;
    numbering_info: ReceivingNumberingInfo;
}

export type TabId =
    | "home"
    | "settings"
    | "query"
    | "numberLocks"
    | "report1"
    | "report2"
    | "report3"
    | "history";
