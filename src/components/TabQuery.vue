<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "../composables/useToast";
import { formatMoney, formatPeriodLabel } from "../lib/format";
import type { DbConfig, PreviewData } from "../lib/types";
import {
    AlertTriangle,
    CalendarDays,
    Database,
    FolderOpen,
    Package,
    Pill,
    Search,
    XCircle,
} from "lucide-vue-next";

const toast = useToast();

const props = defineProps<{
    dbConfig: DbConfig;
    startDateHtml: string;
    endDateHtml: string;
    outputDir: string;
    previewData: PreviewData | null;
    round: number;
}>();

const emit = defineEmits<{
    (e: "update:startDateHtml", v: string): void;
    (e: "update:endDateHtml", v: string): void;
    (e: "update:outputDir", v: string): void;
    (e: "update:previewData", v: PreviewData | null): void;
    (e: "update:round", v: number): void;
    (e: "navigate", tab: "settings"): void;
}>();

const loading = ref(false);
const error = ref("");
// Invalidates in-flight fetches when the date range changes.
let fetchGen = 0;

watch(
    () => [props.startDateHtml, props.endDateHtml],
    () => {
        fetchGen++;
        loading.value = false;
    }
);

function toApiDate(html: string): string {
    return html.replace(/-/g, "");
}

const periodLabel = computed(() =>
    formatPeriodLabel(props.startDateHtml, props.endDateHtml)
);

const activePeriod = computed(() => {
    if (!props.startDateHtml || !props.endDateHtml) return 0;
    // Only a full match counts, so the highlighted preset always describes
    // the exact range that will be fetched.
    if (props.startDateHtml.substring(0, 7) !== props.endDateHtml.substring(0, 7)) return 0;
    const sd = parseInt(props.startDateHtml.substring(8, 10), 10);
    const ed = parseInt(props.endDateHtml.substring(8, 10), 10);
    const y = parseInt(props.startDateHtml.substring(0, 4), 10);
    const m = parseInt(props.startDateHtml.substring(5, 7), 10);
    const last = new Date(y, m, 0).getDate();
    if (sd === 1 && ed === 10) return 1;
    if (sd === 11 && ed === 20) return 2;
    if (sd === 21 && ed === last) return 3;
    return 0;
});

function pad(n: number): string {
    return String(n).padStart(2, "0");
}

/** Fill the 10-day disbursement period (งวด) for the selected month. */
function applyPeriod(part: 1 | 2 | 3) {
    const now = new Date();
    const base = props.startDateHtml || `${now.getFullYear()}-${pad(now.getMonth() + 1)}-01`;
    const y = parseInt(base.substring(0, 4), 10);
    const m = parseInt(base.substring(5, 7), 10);
    const last = new Date(y, m, 0).getDate();
    const from = part === 1 ? 1 : part === 2 ? 11 : 21;
    const to = part === 1 ? 10 : part === 2 ? 20 : last;
    emit("update:startDateHtml", `${y}-${pad(m)}-${pad(from)}`);
    emit("update:endDateHtml", `${y}-${pad(m)}-${pad(to)}`);
}

const isDbReady = computed(
    () => props.dbConfig.host.trim() !== "" && props.dbConfig.username.trim() !== ""
);
const canFetch = computed(
    () => isDbReady.value && props.startDateHtml !== "" && props.endDateHtml !== ""
);

const drugCount = computed(
    () => props.previewData?.invoices.filter((i) => i.category === "ยา").length ?? 0
);
const supplyCount = computed(
    () =>
        props.previewData?.invoices.filter((i) => i.category === "วัสดุเภสัชกรรม")
            .length ?? 0
);

async function browseFolder() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "เลือกโฟลเดอร์สำหรับบันทึกไฟล์รายงาน",
        });
        if (selected && typeof selected === "string") {
            emit("update:outputDir", selected);
        }
    } catch (e) {
        console.error("Browse folder error:", e);
    }
}

async function fetchData() {
    if (!canFetch.value) return;
    const gen = ++fetchGen;
    loading.value = true;
    error.value = "";
    emit("update:previewData", null);
    try {
        const data = await invoke<PreviewData>("fetch_preview", {
            config: { ...props.dbConfig },
            dateFrom: toApiDate(props.startDateHtml),
            dateTo: toApiDate(props.endDateHtml),
        });
        if (gen !== fetchGen) return;
        emit("update:previewData", data);
        if (data.row_count === 0) {
            toast.warning("ไม่พบข้อมูล", "ไม่พบรายการบิลในช่วงวันที่ที่เลือก");
        } else {
            toast.success(
                "ดึงข้อมูลสำเร็จ",
                `พบ ${data.row_count} รายการ ยอดรวม ${formatMoney(data.total_amount)} บาท`
            );
        }
    } catch (e) {
        if (gen !== fetchGen) return;
        error.value = String(e);
        toast.error("ดึงข้อมูลล้มเหลว", String(e));
    } finally {
        if (gen === fetchGen) {
            loading.value = false;
        }
    }
}
</script>

<template>
<div class="query-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">ดึงข้อมูล</h2>
        </div>
    </div>

    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><CalendarDays :size="16" /> ช่วงวันที่และรอบการทำงาน</div>
            </div>
        </div>

        <div class="form-grid-2">
            <div class="form-group">
                <label for="query-from">วันที่เริ่มต้น</label>
                <input id="query-from" type="date" :value="startDateHtml"
                    @input="emit('update:startDateHtml', ($event.target as HTMLInputElement).value)" />
            </div>
            <div class="form-group">
                <label for="query-to">วันที่สิ้นสุด</label>
                <input id="query-to" type="date" :value="endDateHtml"
                    @input="emit('update:endDateHtml', ($event.target as HTMLInputElement).value)" />
            </div>
        </div>

        <div class="period-presets">
            <span class="preset-label">เลือกงวดอัตโนมัติ</span>
            <button type="button" class="btn btn-sm"
                :class="activePeriod === 1 ? 'btn-primary' : 'btn-secondary'"
                @click="applyPeriod(1)">งวด 1 (1-10)</button>
            <button type="button" class="btn btn-sm"
                :class="activePeriod === 2 ? 'btn-primary' : 'btn-secondary'"
                @click="applyPeriod(2)">งวด 2 (11-20)</button>
            <button type="button" class="btn btn-sm"
                :class="activePeriod === 3 ? 'btn-primary' : 'btn-secondary'"
                @click="applyPeriod(3)">งวด 3 (21-สิ้นเดือน)</button>
        </div>

        <div v-if="periodLabel" class="query-period">
            <span class="badge badge-brand"><CalendarDays :size="12" /> {{ periodLabel }}</span>
        </div>

        <hr class="card-divider" />

        <div class="form-grid-2">
            <div class="form-group">
                <label for="query-round">รอบที่</label>
                <input id="query-round" type="number" min="1" max="99" :value="round"
                    @input="emit('update:round', parseInt(($event.target as HTMLInputElement).value) || 1)" />
                <span class="field-hint">เช่น 1, 2, 3</span>
            </div>
            <div class="form-group">
                <label for="query-dir">โฟลเดอร์จัดเก็บไฟล์รายงาน</label>
                <div class="input-group">
                    <input id="query-dir" type="text" :value="outputDir"
                        @input="emit('update:outputDir', ($event.target as HTMLInputElement).value)"
                        placeholder="ปล่อยว่าง = โฟลเดอร์ปัจจุบัน" />
                    <button type="button" class="btn btn-secondary" @click="browseFolder">
                        <FolderOpen :size="15" /> เลือก
                    </button>
                </div>
            </div>
        </div>

        <div v-if="!isDbReady" class="callout callout-warn status-stack">
            <AlertTriangle :size="15" />
            <div class="callout-body">
                <span class="callout-title">ยังไม่ได้ตั้งค่าฐานข้อมูล</span>
            </div>
            <button class="btn btn-secondary btn-sm" @click="emit('navigate', 'settings')">
                ไปที่ตั้งค่า
            </button>
        </div>

        <div class="actions actions-row">
            <button class="btn btn-primary btn-lg" :disabled="!canFetch || loading" @click="fetchData">
                <span v-if="loading" class="spinner"></span>
                <Database v-else :size="16" />
                {{ loading ? "กำลังโหลดข้อมูล..." : "ดึงข้อมูล" }}
            </button>
        </div>

        <div v-if="error" class="status-msg status-error status-stack">
            <XCircle :size="15" /> {{ error }}
        </div>
    </div>

    <!-- Results -->
    <div v-if="previewData" class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><Database :size="16" /> ผลการดึงข้อมูล</div>
            </div>
            <span v-if="previewData.row_count > 0" class="badge badge-success">
                พร้อมสร้างรายงาน
            </span>
        </div>

        <div class="stat-grid">
            <div class="summary-stat">
                <span class="summary-stat-label">จำนวนรายการ</span>
                <span class="summary-stat-value">{{ previewData.row_count }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">ยอดรวมทั้งหมด</span>
                <span class="summary-stat-value money">{{ formatMoney(previewData.total_amount) }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label"><Pill :size="12" /> ยา</span>
                <span class="summary-stat-value">{{ drugCount }} ใบ</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label"><Package :size="12" /> วัสดุเภสัชกรรม</span>
                <span class="summary-stat-value">{{ supplyCount }} ใบ</span>
            </div>
        </div>

        <div v-if="previewData.row_count === 0" class="callout callout-warn card-body">
            <AlertTriangle :size="15" />
            <div class="callout-body">
                <span class="callout-title">ไม่พบข้อมูลในช่วงวันที่นี้</span>
            </div>
        </div>

        <div v-else class="table-wrap card-body">
            <table class="data-table">
                <thead>
                    <tr>
                        <th class="text-center">#</th>
                        <th>วันที่รับของ</th>
                        <th>เลขที่เอกสาร</th>
                        <th>รหัสบริษัท</th>
                        <th>ชื่อบริษัท</th>
                        <th class="text-center">ประเภท</th>
                        <th class="text-right">จำนวนเงิน (บาท)</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(inv, idx) in previewData.invoices" :key="idx">
                        <td class="text-center num">{{ idx + 1 }}</td>
                        <td>{{ inv.receive_date }}</td>
                        <td><code>{{ inv.invoice_no }}</code></td>
                        <td><code>{{ inv.company_keyword }}</code></td>
                        <td>{{ inv.company_name }}</td>
                        <td class="text-center">
                            <span :class="['cat-badge', inv.category === 'ยา' ? 'cat-drug' : 'cat-supply']">
                                {{ inv.category }}
                            </span>
                        </td>
                        <td class="text-right money-cell">{{ formatMoney(inv.total_cost) }}</td>
                    </tr>
                </tbody>
                <tfoot>
                    <tr>
                        <td colspan="6" class="text-right">รวมทั้งสิ้น</td>
                        <td class="text-right">{{ formatMoney(previewData.total_amount) }}</td>
                    </tr>
                </tfoot>
            </table>
        </div>
    </div>

    <div v-else-if="!loading" class="card">
        <div class="empty-state">
            <div class="empty-icon"><Search :size="40" stroke-width="1.5" /></div>
            <div class="empty-title">ยังไม่ได้ดึงข้อมูล</div>
            <p class="empty-desc">เลือกช่วงวันที่ด้านบนแล้วกด "ดึงข้อมูล"</p>
        </div>
    </div>
</div>
</template>

<style scoped>
.period-presets {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    flex-wrap: wrap;
    margin-top: var(--sp-4);
}

.preset-label {
    font-size: var(--fs-xs);
    font-weight: 600;
    color: var(--c-text-light);
    margin-right: var(--sp-1);
}

.query-period {
    margin-top: var(--sp-3);
}
</style>
