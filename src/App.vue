<script setup lang="ts">
import { ref, reactive, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

// --- Types matching Rust backend ---
interface DbConfig {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
}

interface InvoiceRow {
    invoice_no: string;
    vendor_code: string;
    company_name: string;
    company_keyword: string;
    total_cost: number;
    receive_date: string;
    category: string;
}

interface GenerateParams {
    db_config: DbConfig;
    start_date: string;
    end_date: string;
    round: number;
    budget_total: number;
    previous_balance: number;
    start_po_no: number;
    start_reg_no: string;
    start_running: number;
    output_dir: string;
    approval_date: string | null;
}

interface PreviewData {
    invoices: InvoiceRow[];
    total_amount: number;
    row_count: number;
}

interface GenerateResult {
    files: string[];
    total_rows: number;
    total_amount: number;
}

// --- State ---
type TabName = "settings" | "generate" | "preview";
const activeTab = ref<TabName>("settings");

const dbConfig = reactive<DbConfig>({
    host: "localhost",
    port: 1433,
    database: "INVS",
    username: "",
    password: "",
});

const connectionStatus = ref<"idle" | "testing" | "success" | "error">("idle");
const connectionMessage = ref("");

const generateForm = reactive({
    startDate: "",
    endDate: "",
    round: 1,
    budgetTotal: 5843812.60,
    previousBalance: 5843812.60,
    startPoNo: 1,
    startRegNo: "69ภ1",
    startRunning: 0,
    outputDir: "",
    approvalDate: "",
});

const isGenerating = ref(false);
const generateMessage = ref("");
const generateStatus = ref<"idle" | "success" | "error">("idle");
const generateResult = ref<GenerateResult | null>(null);

const isPreviewing = ref(false);
const previewData = ref<PreviewData | null>(null);
const previewError = ref("");

// --- Computed ---
const monthNames = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", "พฤษภาคม", "มิถุนายน",
    "กรกฎาคม", "สิงหาคม", "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม",
];

const periodLabel = computed(() => {
    if (!generateForm.startDate || !generateForm.endDate) {
        return `รอบ ${generateForm.round}`;
    }
    // Parse dates in YYYY-MM-DD format and display in Thai format
    const startYear = parseInt(generateForm.startDate.substring(0, 4)) + 543;
    const startMonth = parseInt(generateForm.startDate.substring(5, 7));
    const startDay = parseInt(generateForm.startDate.substring(8, 10));
    const endYear = parseInt(generateForm.endDate.substring(0, 4)) + 543;
    const endMonth = parseInt(generateForm.endDate.substring(5, 7));
    const endDay = parseInt(generateForm.endDate.substring(8, 10));

    const startMonthName = monthNames[startMonth - 1] || "";
    const endMonthName = monthNames[endMonth - 1] || "";

    if (startMonth === endMonth && startYear === endYear) {
        return `${startDay}-${endDay} ${startMonthName} ${startYear} รอบ ${generateForm.round}`;
    } else {
        return `${startDay} ${startMonthName} - ${endDay} ${endMonthName} ${endYear} รอบ ${generateForm.round}`;
    }
});

const isDbConfigValid = computed(() => {
    return (
        dbConfig.host.trim() !== "" &&
        dbConfig.port > 0 &&
        dbConfig.database.trim() !== "" &&
        dbConfig.username.trim() !== "" &&
        dbConfig.password.trim() !== ""
    );
});

// Separate cover-letter files from summary files
const coverLetterFiles = computed(() => {
    if (!generateResult.value) return [];
    return generateResult.value.files.filter(f => f.includes("เบิกยาปะหน้า"));
});

const summaryFiles = computed(() => {
    if (!generateResult.value) return [];
    return generateResult.value.files.filter(f => !f.includes("เบิกยาปะหน้า"));
});

// --- Methods ---
function formatNumber(n: number): string {
    return n.toLocaleString("th-TH", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

// Extract filename from full path
function fileName(path: string): string {
    return path.split(/[\\/]/).pop() ?? path;
}

async function testConnection() {
    if (!isDbConfigValid.value) {
        connectionMessage.value = "กรุณากรอกข้อมูลการเชื่อมต่อให้ครบถ้วน";
        connectionStatus.value = "error";
        return;
    }
    connectionStatus.value = "testing";
    connectionMessage.value = "กำลังทดสอบการเชื่อมต่อ...";
    try {
        const msg = await invoke<string>("test_connection", {
            config: { ...dbConfig },
        });
        connectionStatus.value = "success";
        connectionMessage.value = msg;
    } catch (e) {
        connectionStatus.value = "error";
        connectionMessage.value = String(e);
    }
}

function buildParams(): GenerateParams {
    return {
        db_config: { ...dbConfig },
        start_date: generateForm.startDate,
        end_date: generateForm.endDate,
        round: generateForm.round,
        budget_total: generateForm.budgetTotal,
        previous_balance: generateForm.previousBalance,
        start_po_no: generateForm.startPoNo,
        start_reg_no: generateForm.startRegNo,
        start_running: generateForm.startRunning,
        output_dir: generateForm.outputDir.trim() || ".",
        approval_date: generateForm.approvalDate.trim() || null,
    };
}

async function previewInvoices() {
    if (!isDbConfigValid.value) {
        previewError.value = "กรุณาตั้งค่าการเชื่อมต่อฐานข้อมูลก่อน";
        return;
    }
    isPreviewing.value = true;
    previewError.value = "";
    previewData.value = null;
    try {
        const data = await invoke<PreviewData>("preview_data", {
            params: buildParams(),
        });
        previewData.value = data;
        activeTab.value = "preview";
    } catch (e) {
        previewError.value = String(e);
    } finally {
        isPreviewing.value = false;
    }
}

async function generateReports() {
    if (!isDbConfigValid.value) {
        generateMessage.value = "กรุณาตั้งค่าการเชื่อมต่อฐานข้อมูลก่อน";
        generateStatus.value = "error";
        return;
    }
    isGenerating.value = true;
    generateMessage.value = "กำลังดึงข้อมูลและสร้างรายงาน...";
    generateStatus.value = "idle";
    generateResult.value = null;
    try {
        const result = await invoke<GenerateResult>("generate_reports", {
            params: buildParams(),
        });
        generateResult.value = result;
        generateStatus.value = "success";
        generateMessage.value = `สร้างรายงานสำเร็จ! (${result.total_rows} รายการ, รวม ${formatNumber(result.total_amount)} บาท) — ${result.files.length} ไฟล์ PDF`;
    } catch (e) {
        generateStatus.value = "error";
        generateMessage.value = String(e);
    } finally {
        isGenerating.value = false;
    }
}
</script>

<template>
    <div class="app-root">
        <!-- Header -->
        <header class="app-header">
            <div class="header-content">
                <h1 class="app-title">
                    <span class="title-icon">📋</span>
                    Swift Bill
                </h1>
                <p class="app-subtitle">ระบบสร้างรายงานเบิกจ่ายยาและเวชภัณฑ์ — โรงพยาบาลสระโบสถ์</p>
            </div>
        </header>

        <!-- Tab Navigation -->
        <nav class="tab-nav">
            <button :class="['tab-btn', { active: activeTab === 'settings' }]" @click="activeTab = 'settings'">
                ⚙️ ตั้งค่าฐานข้อมูล
            </button>
            <button :class="['tab-btn', { active: activeTab === 'generate' }]" @click="activeTab = 'generate'">
                📄 สร้างรายงาน
            </button>
            <button :class="['tab-btn', { active: activeTab === 'preview' }]" @click="activeTab = 'preview'">
                👁️ ดูข้อมูล
                <span v-if="previewData" class="badge">{{ previewData.row_count }}</span>
            </button>
        </nav>

        <!-- Main Content -->
        <main class="main-content">

            <!-- ===== SETTINGS TAB ===== -->
            <section v-if="activeTab === 'settings'" class="tab-panel">
                <div class="card">
                    <h2 class="card-title">🔌 การเชื่อมต่อ SQL Server (INVS)</h2>
                    <p class="card-desc">กรอกข้อมูลเซิร์ฟเวอร์ INVS เพื่อเชื่อมต่อผ่าน TDS Protocol (ไม่ต้องติดตั้ง ODBC
                        Driver)</p>

                    <div class="form-grid">
                        <div class="form-group">
                            <label for="db-host">Host / IP Address</label>
                            <input id="db-host" v-model="dbConfig.host" type="text" placeholder="เช่น 192.168.1.100" />
                        </div>
                        <div class="form-group">
                            <label for="db-port">Port</label>
                            <input id="db-port" v-model.number="dbConfig.port" type="number" placeholder="1433" />
                        </div>
                        <div class="form-group">
                            <label for="db-name">Database Name</label>
                            <input id="db-name" v-model="dbConfig.database" type="text" placeholder="INVS" />
                        </div>
                        <div class="form-group">
                            <label for="db-user">Username</label>
                            <input id="db-user" v-model="dbConfig.username" type="text" placeholder="sa" />
                        </div>
                        <div class="form-group">
                            <label for="db-pass">Password</label>
                            <input id="db-pass" v-model="dbConfig.password" type="password" placeholder="••••••••" />
                        </div>
                    </div>

                    <div class="form-actions">
                        <button class="btn btn-primary" :disabled="connectionStatus === 'testing' || !isDbConfigValid"
                            @click="testConnection">
                            <span v-if="connectionStatus === 'testing'" class="spinner"></span>
                            {{ connectionStatus === "testing" ? "กำลังทดสอบ..." : "🔗 ทดสอบการเชื่อมต่อ" }}
                        </button>
                    </div>

                    <div v-if="connectionMessage" :class="['status-msg', {
                        'status-success': connectionStatus === 'success',
                        'status-error': connectionStatus === 'error',
                        'status-testing': connectionStatus === 'testing',
                    }]">
                        <span v-if="connectionStatus === 'success'">✅</span>
                        <span v-else-if="connectionStatus === 'error'">❌</span>
                        <span v-else>⏳</span>
                        {{ connectionMessage }}
                    </div>
                </div>
            </section>

            <!-- ===== GENERATE TAB ===== -->
            <section v-if="activeTab === 'generate'" class="tab-panel">
                <div class="card">
                    <h2 class="card-title">📄 ตั้งค่าการสร้างรายงาน</h2>
                    <p class="card-desc">กำหนดช่วงเวลา รอบ งบประมาณ และเลขที่เอกสารเริ่มต้น</p>

                    <!-- ─── Period + Round ─── -->
                    <div class="section-label">📅 ช่วงเวลาและรอบการทำงาน</div>
                    <div class="form-grid">
                        <div class="form-group">
                            <label for="gen-start-date">วันที่เริ่มต้น</label>
                            <input id="gen-start-date" v-model="generateForm.startDate" type="date" />
                        </div>
                        <div class="form-group">
                            <label for="gen-end-date">วันที่สิ้นสุด</label>
                            <input id="gen-end-date" v-model="generateForm.endDate" type="date" />
                        </div>
                        <div class="form-group">
                            <label for="gen-round">รอบที่ (ภายในช่วงวันที่)</label>
                            <input id="gen-round" v-model.number="generateForm.round" type="number" min="1" max="99"
                                placeholder="1" />
                        </div>
                    </div>

                    <div class="period-badge">{{ periodLabel }}</div>

                    <div class="info-box">
                        <strong>💡 การทำงานเป็นรอบ:</strong>
                        เช่น รอบ 1 มีบิล 10 ใบ → รอบ 2 มีเพิ่มอีก 20 ใบ โดยเลขทะเบียน เลขขอซื้อ
                        และยอดงบประมาณคงเหลือจะต่อเนื่องจากรอบก่อนหน้าโดยอัตโนมัติ
                    </div>

                    <!-- ─── Budget ─── -->
                    <div class="section-label">💰 งบประมาณ</div>
                    <div class="form-grid">
                        <div class="form-group">
                            <label for="gen-budget">ยอดเงินจัดสรรทั้งหมด (บาท)</label>
                            <input id="gen-budget" v-model.number="generateForm.budgetTotal" type="number" step="0.01"
                                placeholder="5843812.60" />
                        </div>
                        <div class="form-group">
                            <label for="gen-balance">ยอดคงเหลือก่อนรอบนี้ (บาท)</label>
                            <input id="gen-balance" v-model.number="generateForm.previousBalance" type="number"
                                step="0.01" placeholder="ยอดที่เหลือจากรอบที่แล้ว" />
                            <span class="field-hint">ป้อนยอดคงเหลือสุดท้ายจากรอบก่อนหน้า</span>
                        </div>
                    </div>

                    <!-- ─── Document Numbers ─── -->
                    <div class="section-label">🔢 เลขที่เอกสารเริ่มต้น (ต่อจากรอบก่อน)</div>
                    <div class="form-grid">
                        <div class="form-group">
                            <label for="gen-po">เลขขอซื้อ / PO เริ่มต้น</label>
                            <input id="gen-po" v-model.number="generateForm.startPoNo" type="number"
                                placeholder="253" />
                            <span class="field-hint">เลขแรกของรอบนี้ (รอบก่อนจบที่เท่าไร + 1)</span>
                        </div>
                        <div class="form-group">
                            <label for="gen-reg">เลขทะเบียนคุมเริ่มต้น</label>
                            <input id="gen-reg" v-model="generateForm.startRegNo" type="text" placeholder="69ภ12" />
                            <span class="field-hint">เช่น 69ภ12, 69ว5</span>
                        </div>
                        <div class="form-group">
                            <label for="gen-running">ลำดับในทะเบียน (0–9)</label>
                            <input id="gen-running" v-model.number="generateForm.startRunning" type="number" min="0"
                                max="9" placeholder="0" />
                            <span class="field-hint">ลำดับที่เริ่มในเล่มทะเบียน (ถ้าเป็นเล่มใหม่ใส่ 0)</span>
                        </div>
                    </div>

                    <!-- ─── Output ─── -->
                    <div class="section-label">📁 ไฟล์ผลลัพธ์ (PDF)</div>
                    <div class="form-grid">
                        <div class="form-group full-width">
                            <label for="gen-output">โฟลเดอร์บันทึกไฟล์</label>
                            <input id="gen-output" v-model="generateForm.outputDir" type="text"
                                placeholder="เช่น C:\Reports หรือ /Users/me/Documents (ปล่อยว่างจะสร้างโฟลเดอร์ 'output' ในไดเรกทอรีปัจจุบัน)" />
                            <span class="field-hint">ระบบจะสร้างโฟลเดอร์ 'output'
                                ภายในโฟลเดอร์ที่ระบุโดยอัตโนมัติ</span>
                        </div>
                        <div class="form-group">
                            <label for="gen-approval">วันที่ขออนุมัติ (แสดงบนเอกสาร)</label>
                            <input id="gen-approval" v-model="generateForm.approvalDate" type="text"
                                placeholder="เช่น 6 พ.ย. 68" />
                        </div>
                    </div>

                    <!-- ─── Actions ─── -->
                    <div class="form-actions">
                        <button class="btn btn-secondary" :disabled="isPreviewing || !isDbConfigValid"
                            @click="previewInvoices">
                            <span v-if="isPreviewing" class="spinner"></span>
                            {{ isPreviewing ? "กำลังโหลด..." : "👁️ ดูตัวอย่างข้อมูล" }}
                        </button>

                        <button class="btn btn-primary btn-large" :disabled="isGenerating || !isDbConfigValid"
                            @click="generateReports">
                            <span v-if="isGenerating" class="spinner"></span>
                            {{ isGenerating ? "กำลังสร้าง PDF..." : "🚀 สร้างรายงาน PDF ทั้งหมด" }}
                        </button>
                    </div>

                    <!-- Preview error -->
                    <div v-if="previewError" class="status-msg status-error">
                        ❌ {{ previewError }}
                    </div>

                    <!-- Generate status -->
                    <div v-if="generateMessage" :class="['status-msg', {
                        'status-success': generateStatus === 'success',
                        'status-error': generateStatus === 'error',
                    }]">
                        <span v-if="generateStatus === 'success'">✅</span>
                        <span v-else-if="generateStatus === 'error'">❌</span>
                        <span v-else>⏳</span>
                        {{ generateMessage }}
                    </div>

                    <!-- Result Details -->
                    <div v-if="generateResult" class="result-card">
                        <h3>📂 ไฟล์ PDF ที่สร้างแล้ว</h3>

                        <div class="result-section">
                            <div class="result-section-title">📊 รายงานสรุป</div>
                            <ul class="file-list">
                                <li v-for="f in summaryFiles" :key="f">
                                    📄 <code>{{ fileName(f) }}</code>
                                    <span class="file-path-hint">{{ f }}</span>
                                </li>
                            </ul>
                        </div>

                        <div class="result-section">
                            <div class="result-section-title">📝 เอกสารเบิกยาปะหน้า (รวมเป็น 1 ไฟล์)</div>
                            <ul class="file-list">
                                <li v-for="f in coverLetterFiles" :key="f">
                                    📄 <code>{{ fileName(f) }}</code>
                                    <span class="file-path-hint">{{ f }}</span>
                                </li>
                            </ul>
                        </div>

                        <div class="result-summary">
                            <span class="result-stat">📦 {{ generateResult.total_rows }} รายการ</span>
                            <span class="result-stat">💰 {{ formatNumber(generateResult.total_amount) }} บาท</span>
                            <span class="result-stat">📄 {{ generateResult.files.length }} ไฟล์ PDF</span>
                        </div>
                    </div>
                </div>
            </section>

            <!-- ===== PREVIEW TAB ===== -->
            <section v-if="activeTab === 'preview'" class="tab-panel">
                <div class="card">
                    <h2 class="card-title">👁️ ดูข้อมูลจากฐานข้อมูล INVS</h2>

                    <div v-if="!previewData" class="empty-state">
                        <p>🔍 ยังไม่มีข้อมูล</p>
                        <p class="hint">ไปที่แท็บ "สร้างรายงาน" แล้วกดปุ่ม "ดูตัวอย่างข้อมูล"</p>
                    </div>

                    <div v-else>
                        <div class="preview-summary">
                            <div class="summary-item">
                                <span class="summary-label">จำนวนรายการ</span>
                                <span class="summary-value">{{ previewData.row_count }} รายการ</span>
                            </div>
                            <div class="summary-item">
                                <span class="summary-label">ยอดรวมทั้งหมด</span>
                                <span class="summary-value">{{ formatNumber(previewData.total_amount) }} บาท</span>
                            </div>
                        </div>

                        <div class="table-wrapper">
                            <table class="data-table">
                                <thead>
                                    <tr>
                                        <th>#</th>
                                        <th>วันที่รับของ</th>
                                        <th>เลขที่เอกสาร</th>
                                        <th>รหัสบริษัท</th>
                                        <th>ชื่อบริษัท</th>
                                        <th>ประเภท</th>
                                        <th class="text-right">จำนวนเงิน (บาท)</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr v-for="(inv, idx) in previewData.invoices" :key="idx">
                                        <td class="text-center">{{ idx + 1 }}</td>
                                        <td>{{ inv.receive_date }}</td>
                                        <td><code>{{ inv.invoice_no }}</code></td>
                                        <td><code>{{ inv.vendor_code }}</code></td>
                                        <td>{{ inv.company_name }}</td>
                                        <td>
                                            <span
                                                :class="['cat-badge', inv.category === 'ยา' ? 'cat-drug' : 'cat-material']">
                                                {{ inv.category }}
                                            </span>
                                        </td>
                                        <td class="text-right">{{ formatNumber(inv.total_cost) }}</td>
                                    </tr>
                                </tbody>
                                <tfoot>
                                    <tr>
                                        <td colspan="6" class="text-right"><strong>รวมทั้งสิ้น</strong></td>
                                        <td class="text-right"><strong>{{ formatNumber(previewData.total_amount)
                                        }}</strong></td>
                                    </tr>
                                </tfoot>
                            </table>
                        </div>
                    </div>
                </div>
            </section>

        </main>

        <!-- Footer -->
        <footer class="app-footer">
            <p>Swift Bill v0.2.0 — ระบบสร้างรายงานเบิกจ่ายยาและเวชภัณฑ์ | Tauri + Vue 3 + Rust</p>
        </footer>
    </div>
</template>

<style>
*,
*::before,
*::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

:root {
    --primary: #1a6fc4;
    --primary-dark: #155aa0;
    --primary-light: #e8f0fb;
    --secondary: #6c757d;
    --success: #198754;
    --success-bg: #d1e7dd;
    --error: #dc3545;
    --error-bg: #f8d7da;
    --warn: #e07d10;
    --warn-bg: #fff3cd;
    --bg: #f4f6fa;
    --card-bg: #ffffff;
    --border: #dee2e6;
    --text: #212529;
    --text-muted: #6c757d;
    --radius: 10px;
    --shadow: 0 2px 12px rgba(0, 0, 0, 0.07);
    --font: 'Segoe UI', 'Noto Sans Thai', 'TH SarabunPSK', Arial, sans-serif;
}

body {
    font-family: var(--font);
    background: var(--bg);
    color: var(--text);
    font-size: 14px;
    line-height: 1.6;
}

.app-root {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}

/* ── Header ── */
.app-header {
    background: linear-gradient(135deg, var(--primary) 0%, var(--primary-dark) 100%);
    color: #fff;
    padding: 16px 24px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.header-content {
    max-width: 1100px;
    margin: 0 auto;
}

.app-title {
    font-size: 22px;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 8px;
}

.title-icon {
    font-size: 26px;
}

.app-subtitle {
    font-size: 13px;
    opacity: 0.85;
    margin-top: 2px;
}

/* ── Tab nav ── */
.tab-nav {
    background: var(--card-bg);
    border-bottom: 2px solid var(--border);
    display: flex;
    gap: 4px;
    padding: 0 24px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
}

.tab-btn {
    background: none;
    border: none;
    padding: 12px 20px;
    font-size: 14px;
    font-family: var(--font);
    cursor: pointer;
    color: var(--text-muted);
    border-bottom: 3px solid transparent;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: -2px;
}

.tab-btn:hover {
    color: var(--primary);
    background: var(--primary-light);
}

.tab-btn.active {
    color: var(--primary);
    border-bottom-color: var(--primary);
    font-weight: 600;
}

.badge {
    background: var(--primary);
    color: #fff;
    border-radius: 12px;
    padding: 1px 7px;
    font-size: 11px;
    font-weight: 700;
}

/* ── Main content ── */
.main-content {
    flex: 1;
    padding: 24px;
    max-width: 1100px;
    margin: 0 auto;
    width: 100%;
}

.tab-panel {
    animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(4px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

/* ── Card ── */
.card {
    background: var(--card-bg);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: 28px 32px;
    border: 1px solid var(--border);
}

.card-title {
    font-size: 18px;
    font-weight: 700;
    color: var(--primary-dark);
    margin-bottom: 6px;
}

.card-desc {
    color: var(--text-muted);
    font-size: 13px;
    margin-bottom: 20px;
}

/* ── Section label ── */
.section-label {
    font-size: 13px;
    font-weight: 700;
    color: var(--primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 22px 0 12px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--primary-light);
}

/* ── Form ── */
.form-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 16px;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.form-group.full-width {
    grid-column: 1 / -1;
}

.form-group label {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.3px;
}

.form-group input,
.form-group select {
    padding: 9px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: 14px;
    font-family: var(--font);
    background: var(--bg);
    color: var(--text);
    transition: border-color 0.15s, box-shadow 0.15s;
}

.form-group input:focus,
.form-group select:focus {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px rgba(26, 111, 196, 0.12);
}

.form-group input::placeholder {
    color: #b0b7c3;
}

.field-hint {
    font-size: 11px;
    color: var(--text-muted);
    line-height: 1.4;
}

/* ── Info box ── */
.info-box {
    background: var(--warn-bg);
    border: 1px solid #f0c060;
    border-radius: 8px;
    padding: 12px 16px;
    font-size: 13px;
    margin: 12px 0;
    color: #5a4000;
    line-height: 1.6;
}

/* ── Period badge ── */
.period-badge {
    display: inline-block;
    background: var(--primary-light);
    color: var(--primary-dark);
    border: 1px solid var(--primary);
    border-radius: 20px;
    padding: 5px 16px;
    font-size: 13px;
    font-weight: 600;
    margin: 10px 0 4px;
}

/* ── Form actions ── */
.form-actions {
    display: flex;
    gap: 12px;
    margin: 24px 0 16px;
    flex-wrap: wrap;
}

/* ── Buttons ── */
.btn {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 10px 20px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-family: var(--font);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
}

.btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-primary {
    background: var(--primary);
    color: #fff;
}

.btn-primary:hover:not(:disabled) {
    background: var(--primary-dark);
    box-shadow: 0 3px 8px rgba(26, 111, 196, 0.3);
}

.btn-secondary {
    background: var(--bg);
    color: var(--secondary);
    border: 1px solid var(--border);
}

.btn-secondary:hover:not(:disabled) {
    background: var(--border);
    color: var(--text);
}

.btn-large {
    padding: 12px 28px;
    font-size: 15px;
}

/* ── Spinner ── */
.spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.4);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
}

.btn-secondary .spinner {
    border-color: rgba(0, 0, 0, 0.15);
    border-top-color: var(--secondary);
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

/* ── Status messages ── */
.status-msg {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 12px 16px;
    border-radius: 8px;
    font-size: 13px;
    margin-top: 12px;
    background: var(--bg);
    border: 1px solid var(--border);
    line-height: 1.5;
}

.status-success {
    background: var(--success-bg);
    border-color: #a3cfbb;
    color: #0a3622;
}

.status-error {
    background: var(--error-bg);
    border-color: #f1aeb5;
    color: #58151c;
}

.status-testing {
    background: var(--warn-bg);
    border-color: #f0c060;
    color: #5a4000;
}

/* ── Result card ── */
.result-card {
    margin-top: 18px;
    background: #f0f7ee;
    border: 1px solid #a3cfbb;
    border-radius: 8px;
    padding: 18px 20px;
}

.result-card h3 {
    font-size: 15px;
    color: var(--success);
    margin-bottom: 14px;
}

.result-section {
    margin-bottom: 14px;
}

.result-section-title {
    font-size: 12px;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.4px;
    margin-bottom: 6px;
}

.file-list {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.file-list li {
    display: flex;
    align-items: baseline;
    gap: 6px;
    flex-wrap: wrap;
}

.file-list code {
    background: #e8f5e9;
    padding: 2px 7px;
    border-radius: 4px;
    font-size: 12px;
    color: #1b5e20;
    word-break: break-all;
}

.file-path-hint {
    font-size: 10px;
    color: var(--text-muted);
    word-break: break-all;
}

.result-summary {
    display: flex;
    gap: 16px;
    margin-top: 12px;
    flex-wrap: wrap;
    border-top: 1px solid #a3cfbb;
    padding-top: 10px;
}

.result-stat {
    font-size: 13px;
    font-weight: 600;
    color: #0a3622;
    background: #c3e6cb;
    padding: 3px 10px;
    border-radius: 12px;
}

/* ── Preview ── */
.empty-state {
    text-align: center;
    padding: 48px 20px;
    color: var(--text-muted);
}

.empty-state .hint {
    font-size: 13px;
    margin-top: 8px;
}

.preview-summary {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
    flex-wrap: wrap;
}

.summary-item {
    background: var(--primary-light);
    border-radius: 8px;
    padding: 12px 18px;
    display: flex;
    flex-direction: column;
    gap: 3px;
}

.summary-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--primary);
    text-transform: uppercase;
    letter-spacing: 0.3px;
}

.summary-value {
    font-size: 20px;
    font-weight: 700;
    color: var(--primary-dark);
}

.table-wrapper {
    overflow-x: auto;
    border-radius: 8px;
    border: 1px solid var(--border);
}

.data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}

.data-table th {
    background: #e8f0fb;
    color: var(--primary-dark);
    font-weight: 700;
    padding: 10px 12px;
    text-align: left;
    border-bottom: 2px solid var(--primary);
    white-space: nowrap;
}

.data-table td {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
}

.data-table tbody tr:hover {
    background: #f0f6ff;
}

.data-table tfoot td {
    background: #f4f6fa;
    font-size: 13px;
    padding: 10px 12px;
    border-top: 2px solid var(--border);
}

.text-right {
    text-align: right;
}

.text-center {
    text-align: center;
}

.cat-badge {
    display: inline-block;
    padding: 2px 9px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 600;
}

.cat-drug {
    background: #cfe2ff;
    color: #084298;
}

.cat-material {
    background: #d1ecf1;
    color: #0c5460;
}

/* ── Footer ── */
.app-footer {
    background: var(--card-bg);
    border-top: 1px solid var(--border);
    padding: 12px 24px;
    text-align: center;
    font-size: 12px;
    color: var(--text-muted);
}

/* ── Dark mode ── */
@media (prefers-color-scheme: dark) {
    :root {
        --bg: #1a1d24;
        --card-bg: #242830;
        --border: #3a3f4b;
        --text: #e2e5ec;
        --text-muted: #8b92a5;
        --primary-light: #1a2a3f;
        --success-bg: #0d2e1a;
        --error-bg: #2e0d0d;
        --warn-bg: #2e2200;
    }

    .form-group input,
    .form-group select {
        background: #1a1d24;
        color: var(--text);
        border-color: var(--border);
    }

    .app-footer {
        background: #1e2128;
    }

    .tab-nav {
        background: var(--card-bg);
        border-bottom-color: var(--border);
    }

    .data-table th {
        background: #1a2a3f;
    }

    .data-table tbody tr:hover {
        background: #1e2535;
    }

    .info-box {
        background: #2e2200;
        border-color: #5a4000;
        color: #f0c060;
    }
}
</style>
