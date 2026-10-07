<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { formatDateTime, formatMoney, formatPeriodLabel } from "../lib/format";
import type { DbConfig, PreviewData, RoundHistoryEntry, TabId } from "../lib/types";
import {
    AlertTriangle,
    ArrowRight,
    CalendarDays,
    Check,
    ClipboardList,
    Database,
    Download,
    History,
    Plug,
    Search,
} from "lucide-vue-next";

const props = defineProps<{
    dbConfig: DbConfig;
    dbConnected: boolean | null;
    previewData: PreviewData | null;
    startDateHtml: string;
    endDateHtml: string;
    round: number;
    historyEntries: RoundHistoryEntry[];
    generated: { report1: boolean; report2: boolean; report3: boolean };
}>();

const emit = defineEmits<{
    (e: "navigate", tab: TabId): void;
    (e: "loadEntry", entry: RoundHistoryEntry): void;
    (e: "connectionStatus", ok: boolean): void;
}>();

const toast = useToast();
const testing = ref(false);

const isConfigured = computed(
    () => props.dbConfig.host.trim() !== "" && props.dbConfig.username.trim() !== ""
);

const dataReady = computed(
    () => props.previewData !== null && props.previewData.row_count > 0
);

const periodLabel = computed(
    () => formatPeriodLabel(props.startDateHtml, props.endDateHtml) || "ยังไม่ได้เลือกช่วงวันที่"
);

const recentEntries = computed(() => props.historyEntries.slice(0, 3));

type StepState = "done" | "ready" | "blocked";

interface FlowStep {
    key: string;
    title: string;
    desc: string;
    state: StepState;
    target: TabId;
    actionLabel: string;
}

const steps = computed<FlowStep[]>(() => [
    {
        key: "query",
        title: "ดึงข้อมูลบิลจาก INVS",
        desc: dataReady.value
            ? `ดึงแล้ว ${props.previewData?.row_count ?? 0} รายการ รวม ${formatMoney(props.previewData?.total_amount ?? 0)} บาท`
            : "เลือกช่วงวันที่และรอบการทำงาน แล้วดึงรายการบิลจากฐานข้อมูล",
        state: dataReady.value ? "done" : isConfigured.value ? "ready" : "blocked",
        target: isConfigured.value ? "query" : "settings",
        actionLabel: !isConfigured.value
            ? "ตั้งค่าฐานข้อมูล"
            : dataReady.value
              ? "ดึงข้อมูลใหม่"
              : "ดึงข้อมูล",
    },
    {
        key: "report1",
        title: "ส่งหนี้เบิกยา",
        desc: "รายการส่งหนี้สินและเอกสารเบิกเงิน ส่งออก Excel หรือ PDF",
        state: props.generated.report1 ? "done" : dataReady.value ? "ready" : "blocked",
        target: "report1",
        actionLabel: props.generated.report1 ? "ดูรายงาน" : "ทำรายงาน",
    },
    {
        key: "report2",
        title: "สรุปรับยา",
        desc: "สรุปยอดรับยาประจำรอบ พร้อมเลขขอซื้อ รายงาน และใบสั่งซื้อ",
        state: props.generated.report2 ? "done" : dataReady.value ? "ready" : "blocked",
        target: "report2",
        actionLabel: props.generated.report2 ? "ดูรายงาน" : "ทำรายงาน",
    },
    {
        key: "report3",
        title: "เบิกยาปะหน้า",
        desc: "หนังสือเบิกยาปะหน้า พร้อมคำนวณงบประมาณคงเหลือต่อบิล",
        state: props.generated.report3 ? "done" : dataReady.value ? "ready" : "blocked",
        target: "report3",
        actionLabel: props.generated.report3 ? "ดูรายงาน" : "ทำรายงาน",
    },
]);

function stepBadge(state: StepState): { text: string; cls: string } {
    if (state === "done") return { text: "เสร็จแล้ว", cls: "badge badge-success" };
    if (state === "ready") return { text: "พร้อมทำ", cls: "badge badge-brand" };
    return { text: "รอข้อมูล", cls: "badge badge-neutral" };
}

async function testConnection() {
    if (!isConfigured.value || testing.value) return;
    testing.value = true;
    try {
        await invoke<string>("test_connection", { config: { ...props.dbConfig } });
        emit("connectionStatus", true);
        toast.success("เชื่อมต่อสำเร็จ", "เชื่อมต่อฐานข้อมูล INVS ได้เรียบร้อย");
    } catch (e) {
        emit("connectionStatus", false);
        toast.error("เชื่อมต่อล้มเหลว", String(e));
    } finally {
        testing.value = false;
    }
}
</script>

<template>
<div class="home-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">ภาพรวม</h2>
            <p class="page-desc">
                ขั้นตอนการทำงานของรอบปัจจุบัน ตั้งแต่ดึงข้อมูลจนสร้างรายงานครบทั้ง 3 ฉบับ
            </p>
        </div>
        <div class="page-actions">
            <button class="btn btn-primary" @click="emit('navigate', isConfigured ? 'query' : 'settings')">
                <Search :size="15" />
                {{ isConfigured ? "เริ่มดึงข้อมูล" : "ตั้งค่าการเชื่อมต่อ" }}
            </button>
        </div>
    </div>

    <!-- Connection required -->
    <div v-if="!isConfigured" class="callout callout-warn">
        <AlertTriangle :size="16" />
        <div class="callout-body">
            <span class="callout-title">ยังไม่ได้ตั้งค่าการเชื่อมต่อฐานข้อมูล</span>
            <span class="callout-desc">
                กรอกข้อมูล SQL Server (INVS) และทดสอบการเชื่อมต่อก่อนเริ่มดึงข้อมูลบิล
            </span>
        </div>
        <button class="btn btn-secondary btn-sm" @click="emit('navigate', 'settings')">
            ไปที่ตั้งค่า
        </button>
    </div>

    <!-- Connection configured but never tested this session -->
    <div v-else-if="dbConnected !== true" class="callout callout-info">
        <Database :size="16" />
        <div class="callout-body">
            <span class="callout-title">ตั้งค่าฐานข้อมูลไว้แล้ว</span>
            <span class="callout-desc">ทดสอบการเชื่อมต่ออีกครั้งเพื่อยืนยันว่าใช้งานได้ก่อนดึงข้อมูล</span>
        </div>
        <button class="btn btn-secondary btn-sm" :disabled="testing" @click="testConnection">
            <span v-if="testing" class="spinner"></span>
            <Plug v-else :size="14" />
            ทดสอบการเชื่อมต่อ
        </button>
    </div>

    <!-- Current round summary -->
    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><CalendarDays :size="16" /> รอบปัจจุบัน</div>
                <div class="card-desc">ค่าที่ใช้กับรายงานทั้ง 3 ฉบับในรอบนี้</div>
            </div>
            <span v-if="dataReady" class="badge badge-success">
                <Check :size="12" /> ข้อมูลพร้อมใช้งาน
            </span>
        </div>

        <div class="preview-summary">
            <div class="summary-stat">
                <span class="summary-stat-label">ช่วงวันที่</span>
                <span class="summary-stat-value period">{{ periodLabel }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">รอบที่</span>
                <span class="summary-stat-value">{{ round }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">รายการที่ดึงแล้ว</span>
                <span class="summary-stat-value">{{ previewData?.row_count ?? 0 }}</span>
            </div>
            <div class="summary-stat">
                <span class="summary-stat-label">ยอดรวม</span>
                <span class="summary-stat-value money">{{ formatMoney(previewData?.total_amount ?? 0) }}</span>
            </div>
        </div>

        <div v-if="!dataReady" class="card-body">
            <div class="callout callout-info">
                <AlertTriangle :size="16" />
                <div class="callout-body">
                    <span class="callout-title">ยังไม่มีข้อมูลของรอบนี้</span>
                    <span class="callout-desc">
                        ไปที่ขั้นตอนที่ 1 เพื่อเลือกช่วงวันที่และดึงรายการบิลก่อนสร้างรายงาน
                    </span>
                </div>
            </div>
        </div>
    </div>

    <!-- Workflow -->
    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><ClipboardList :size="16" /> ขั้นตอนการทำงาน</div>
                <div class="card-desc">ทำตามลำดับ ระบบจะปลดล็อกขั้นถัดไปเมื่อข้อมูลพร้อม</div>
            </div>
        </div>

        <div class="flow-list">
            <div
                v-for="(step, index) in steps"
                :key="step.key"
                class="flow-step"
                :class="step.state"
            >
                <span class="flow-index">
                    <Check v-if="step.state === 'done'" :size="15" />
                    <template v-else>{{ index + 1 }}</template>
                </span>
                <div class="flow-body">
                    <div class="flow-title-row">
                        <span class="flow-title">{{ step.title }}</span>
                        <span :class="stepBadge(step.state).cls">{{ stepBadge(step.state).text }}</span>
                    </div>
                    <p class="flow-desc">{{ step.desc }}</p>
                </div>
                <div class="flow-action">
                    <button
                        class="btn btn-sm"
                        :class="step.state === 'ready' ? 'btn-primary' : 'btn-secondary'"
                        @click="emit('navigate', step.target)"
                    >
                        {{ step.actionLabel }}
                        <ArrowRight :size="13" />
                    </button>
                </div>
            </div>
        </div>
    </div>

    <!-- Recent rounds -->
    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><History :size="16" /> รอบล่าสุด</div>
                <div class="card-desc">โหลดค่า carry-forward จากรอบก่อนไปใช้กับรอบถัดไปได้ทันที</div>
            </div>
            <button class="btn btn-ghost btn-sm" @click="emit('navigate', 'history')">
                ดูประวัติทั้งหมด <ArrowRight :size="13" />
            </button>
        </div>

        <div v-if="recentEntries.length === 0" class="empty-state compact">
            <div class="empty-icon"><History :size="34" stroke-width="1.5" /></div>
            <div class="empty-title">ยังไม่มีประวัติรอบ</div>
            <p class="empty-desc">
                เมื่อสร้างรายงานเสร็จในแต่ละรอบ ให้บันทึกรอบไว้ที่หน้าผลรายงาน
                แล้วรอบถัดไปจะเริ่มต่อจากค่าเดิมได้เลย
            </p>
        </div>

        <ul v-else class="recent-list">
            <li v-for="entry in recentEntries" :key="entry.id" class="recent-item">
                <div class="recent-main">
                    <span class="recent-label">{{ entry.label }}</span>
                    <span class="recent-meta">
                        {{ entry.invoice_count }} บิล · {{ formatMoney(entry.total_amount) }} บาท ·
                        บันทึก {{ formatDateTime(entry.created_at) }}
                    </span>
                </div>
                <button class="btn btn-secondary btn-sm" @click="emit('loadEntry', entry)">
                    <Download :size="13" />
                    โหลดไปใช้รอบถัดไป
                </button>
            </li>
        </ul>
    </div>

    <p class="home-credit">ไฟล์รายงานทั้งหมดถูกบันทึกไว้ในเครื่องนี้ ไม่มีการส่งข้อมูลออกภายนอก</p>
</div>
</template>

<style scoped>
.recent-list {
    list-style: none;
    display: flex;
    flex-direction: column;
}

.recent-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--sp-4);
    padding: 12px 2px;
    border-bottom: 1px solid var(--c-border-soft);
}

.recent-item:last-child {
    border-bottom: none;
    padding-bottom: 0;
}

.recent-main {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
}

.recent-label {
    font-size: var(--fs-body);
    font-weight: 600;
    color: var(--c-text);
}

.recent-meta {
    font-size: var(--fs-xs);
    color: var(--c-text-light);
}

.home-credit {
    text-align: center;
    font-size: var(--fs-xs);
    color: var(--c-text-light);
    padding: var(--sp-2) 0 var(--sp-4);
}
</style>
