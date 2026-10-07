<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import TabHome from "./components/TabHome.vue";
import TabSettings from "./components/TabSettings.vue";
import TabQuery from "./components/TabQuery.vue";
import TabNumberLocks from "./components/TabNumberLocks.vue";
import TabReport1 from "./components/TabReport1.vue";
import TabReport2 from "./components/TabReport2.vue";
import TabReport3 from "./components/TabReport3.vue";
import TabHistory from "./components/TabHistory.vue";
import ToastContainer from "./components/ToastContainer.vue";
import { useToast } from "./composables/useToast";
import type {
    DbConfig,
    PreviewData,
    ReceivingNumberingInfo,
    RoundHistoryEntry,
    TabId,
} from "./lib/types";
import {
    Check,
    History,
    LayoutDashboard,
    Lock,
    Settings2,
} from "lucide-vue-next";

const toast = useToast();

// Shared state

const activeTab = ref<TabId>("home");

const dbConfig = reactive<DbConfig>({
    host: "localhost",
    port: 1433,
    database: "INVS",
    username: "",
    password: "",
});

// Query / shared period state
const startDateHtml = ref(""); // YYYY-MM-DD (HTML date input format)
const endDateHtml = ref("");
const outputDir = ref("");
const previewData = ref<PreviewData | null>(null);

const dateFrom = computed(() => startDateHtml.value.replace(/-/g, ""));
const dateTo = computed(() => endDateHtml.value.replace(/-/g, ""));
// Fiscal year: derived from the start date by default, editable in the report tabs.
const year = ref(0);
watch(
    startDateHtml,
    (value) => {
        year.value = value ? parseInt(value.substring(0, 4)) + 543 : 0;
    },
    { immediate: true }
);
const month = computed(() =>
    startDateHtml.value ? parseInt(startDateHtml.value.substring(5, 7)) : 0
);

// Shared round number (applies to all 3 reports)
const round = ref(1);

// DB connection status (set by TabSettings / TabHome via connectionStatus event)
const dbConnected = ref<boolean | null>(null);

// Per-report unique fields
const r1Form = reactive({ startRegNo: "69ภ1", startRunning: 0 });
const r2Form = reactive({
    startPoNo: 1,
    startPurchaseNo: 1,
    startRegNo: "69ภ1",
    startRunning: 0,
    approvalDate: "",
});
const r3Form = reactive({
    budgetTotal: 5843812.6,
    previousBalance: 5843812.6,
    approvalDate: "",
});

// Per-report completion flags for the overview stepper, reset when the
// underlying data changes so the checklist always reflects reality.
const generated = reactive({ report1: false, report2: false, report3: false });

watch(previewData, () => {
    generated.report1 = false;
    generated.report2 = false;
    generated.report3 = false;
});

const dataLoaded = computed(() => (previewData.value?.row_count ?? 0) > 0);

// History
const historyEntries = ref<RoundHistoryEntry[]>([]);

// Carry-forward results stored from each report tab (for combined history save)
const r2Carry = ref<{
    next_reg_no: string;
    next_running: number;
    next_po_no: number;
    next_purchase_no: number;
} | null>(null);

// App version (single-sourced from the Tauri bundle)
const appVersion = ref("");

// Lifecycle

onMounted(async () => {
    // Load the encrypted connection settings from the OS-keychain-backed store.
    const savedConfig = await invoke<DbConfig | null>("load_db_config").catch(() => null);
    if (savedConfig) {
        Object.assign(dbConfig, savedConfig);
    }
    try {
        historyEntries.value = await invoke<RoundHistoryEntry[]>("load_round_history");
    } catch (_) {
        /* ignore on fresh install */
    }
    try {
        appVersion.value = await getVersion();
    } catch (_) {
        /* version is cosmetic */
    }
});

// History handlers

async function refreshHistory() {
    try {
        historyEntries.value = await invoke<RoundHistoryEntry[]>("load_round_history");
    } catch (_) { }
}

async function saveEntry(entry: RoundHistoryEntry) {
    try {
        await invoke("save_round_entry", { entry });
        await refreshHistory();
        toast.success("บันทึกประวัติสำเร็จ", `บันทึกรอบ ${entry.round} เรียบร้อยแล้ว`);
    } catch (e) {
        toast.error("บันทึกประวัติล้มเหลว", String(e));
    }
}

async function deleteEntry(id: string) {
    try {
        await invoke("delete_round_entry", { id });
        await refreshHistory();
        toast.success("ลบประวัติสำเร็จ", "ลบรายการประวัติเรียบร้อยแล้ว");
    } catch (e) {
        toast.error("ลบประวัติล้มเหลว", String(e));
    }
}

async function saveDbConfig() {
    try {
        await invoke("save_db_config", { config: { ...dbConfig } });
        toast.success("บันทึกการตั้งค่าสำเร็จ", "ข้อมูลการเชื่อมต่อถูกเข้ารหัสและบันทึกไว้ในเครื่องแล้ว");
    } catch (e) {
        toast.error("บันทึกการตั้งค่าล้มเหลว", String(e));
    }
}

function handleConnectionStatus(ok: boolean) {
    dbConnected.value = ok;
    if (ok) {
        toast.success("เชื่อมต่อสำเร็จ", "เชื่อมต่อฐานข้อมูล INVS ได้เรียบร้อย");
    } else {
        toast.error("เชื่อมต่อล้มเหลว", "ไม่สามารถเชื่อมต่อฐานข้อมูลได้ กรุณาตรวจสอบการตั้งค่า");
    }
}

function handleR2Carry(carry: { next_reg_no: string; next_running: number; next_po_no: number; next_purchase_no: number }) {
    r2Carry.value = carry;
}

async function applyHistoryEntry(entry: RoundHistoryEntry) {
    let numberingInfo: ReceivingNumberingInfo;
    try {
        numberingInfo = await invoke<ReceivingNumberingInfo>("normalize_receiving_start", {
            params: {
                fiscal_year: entry.fiscal_year,
                start_po_no: entry.next_po_no,
                start_purchase_no: entry.next_purchase_no ?? 1,
            },
        });
    } catch (e) {
        toast.error("โหลดประวัติล้มเหลว", `ไม่สามารถตรวจสอบเลขล็อกได้: ${String(e)}`);
        return;
    }

    // Pre-fill shared state
    round.value = entry.round + 1;

    // Pre-fill report 1 form
    r1Form.startRegNo = entry.next_reg_no;
    r1Form.startRunning = entry.next_running;

    // Pre-fill report 2 form
    r2Form.startPoNo = numberingInfo.start_po_no;
    r2Form.startPurchaseNo = numberingInfo.start_purchase_no;
    r2Form.startRegNo = entry.next_reg_no;
    r2Form.startRunning = entry.next_running;

    // Pre-fill report 3 form
    r3Form.budgetTotal = entry.budget_total;
    r3Form.previousBalance = entry.remaining_balance;

    // A new round needs a fresh date range, so drop the previous dataset.
    previewData.value = null;

    // Switch to query tab so user can pick the new date range
    activeTab.value = "query";

    if (numberingInfo.skipped_locked_sets.length > 0) {
        toast.info(
            "โหลดประวัติสำเร็จ",
            `โหลดค่า carry-forward จากรอบ ${entry.round} แล้ว - ข้ามเลขล็อก ${numberingInfo.skipped_locked_sets.length} ชุดให้อัตโนมัติ`
        );
        return;
    }

    toast.info(
        "โหลดประวัติสำเร็จ",
        `โหลดค่า carry-forward จากรอบ ${entry.round} แล้ว - พร้อมทำงานรอบ ${entry.round + 1}`
    );
}
</script>

<template>
<div class="app-root">

    <a class="skip-link" href="#main-content">ข้ามไปยังเนื้อหาหลัก</a>

    <!-- ── Sidebar ─────────────────────────────────────────── -->
    <aside class="sidebar">

        <!-- Brand -->
        <div class="sidebar-brand">
            <div class="brand-icon" aria-hidden="true">
                <img src="/swift-bill-icon.svg" alt="" class="brand-icon-img" />
            </div>
            <div class="brand-text">
                <span class="brand-name">Swift Bill</span>
                <span class="brand-sub">โรงพยาบาลสระโบสถ์</span>
            </div>
        </div>

        <!-- Data context chip (shown once data is loaded) -->
        <div class="sidebar-context" v-if="previewData">
            <div class="context-chip">
                <span class="context-dot"></span>
                ข้อมูล {{ previewData.row_count }} รายการ
                <span class="context-round">รอบ {{ round }}</span>
            </div>
        </div>

        <!-- Navigation -->
        <nav class="sidebar-nav" aria-label="เมนูหลัก">
            <button class="nav-item" :class="{ active: activeTab === 'home' }"
                :aria-current="activeTab === 'home' ? 'page' : undefined"
                @click="activeTab = 'home'">
                <LayoutDashboard :size="15" :stroke-width="2" />
                <span class="nav-text">ภาพรวม</span>
            </button>

            <span class="nav-section-label">งานรายเดือน</span>
            <button class="nav-item" :class="{ active: activeTab === 'query' }"
                :aria-current="activeTab === 'query' ? 'page' : undefined"
                @click="activeTab = 'query'">
                <span class="nav-step" :class="{ done: dataLoaded }">
                    <Check v-if="dataLoaded" :size="12" :stroke-width="3" />
                    <template v-else>1</template>
                </span>
                <span class="nav-text">ดึงข้อมูล</span>
            </button>
            <button class="nav-item" :class="{ active: activeTab === 'report1' }"
                :aria-current="activeTab === 'report1' ? 'page' : undefined"
                @click="activeTab = 'report1'">
                <span class="nav-step" :class="{ done: generated.report1 }">
                    <Check v-if="generated.report1" :size="12" :stroke-width="3" />
                    <template v-else>2</template>
                </span>
                <span class="nav-text">ส่งหนี้เบิกยา</span>
            </button>
            <button class="nav-item" :class="{ active: activeTab === 'report2' }"
                :aria-current="activeTab === 'report2' ? 'page' : undefined"
                @click="activeTab = 'report2'">
                <span class="nav-step" :class="{ done: generated.report2 }">
                    <Check v-if="generated.report2" :size="12" :stroke-width="3" />
                    <template v-else>3</template>
                </span>
                <span class="nav-text">สรุปรับยา</span>
            </button>
            <button class="nav-item" :class="{ active: activeTab === 'report3' }"
                :aria-current="activeTab === 'report3' ? 'page' : undefined"
                @click="activeTab = 'report3'">
                <span class="nav-step" :class="{ done: generated.report3 }">
                    <Check v-if="generated.report3" :size="12" :stroke-width="3" />
                    <template v-else>4</template>
                </span>
                <span class="nav-text">เบิกยาปะหน้า</span>
            </button>

            <span class="nav-section-label">เครื่องมือ</span>
            <button class="nav-item" :class="{ active: activeTab === 'numberLocks' }"
                :aria-current="activeTab === 'numberLocks' ? 'page' : undefined"
                @click="activeTab = 'numberLocks'">
                <Lock :size="15" :stroke-width="2" />
                <span class="nav-text">ล็อกเลข</span>
            </button>
            <button class="nav-item" :class="{ active: activeTab === 'history' }"
                :aria-current="activeTab === 'history' ? 'page' : undefined"
                @click="activeTab = 'history'">
                <History :size="15" :stroke-width="2" />
                <span class="nav-text">ประวัติรอบ</span>
            </button>
        </nav>

        <!-- Sidebar footer -->
        <div class="sidebar-footer">
            <button class="nav-item" :class="{ active: activeTab === 'settings' }"
                :aria-current="activeTab === 'settings' ? 'page' : undefined"
                @click="activeTab = 'settings'">
                <Settings2 :size="15" :stroke-width="2" />
                <span class="nav-text">ตั้งค่าฐานข้อมูล</span>
            </button>
            <div class="conn-badge">
                <span class="conn-dot"
                    :class="dbConnected === true ? 'ok' : dbConnected === false ? 'fail' : 'unknown'">
                </span>
                <span class="conn-text">
                    {{ dbConnected === true ? 'INVS เชื่อมต่อแล้ว'
                     : dbConnected === false ? 'เชื่อมต่อไม่สำเร็จ'
                     : 'ยังไม่ได้ทดสอบ' }}
                </span>
            </div>
            <span class="app-version" v-if="appVersion">ภก.สุรเดช · v{{ appVersion }}</span>
        </div>

    </aside>

    <!-- ── Main content area ────────────────────────────────── -->
    <main id="main-content" class="main-area">
        <TabHome v-show="activeTab === 'home'"
            :db-config="dbConfig" :db-connected="dbConnected"
            :preview-data="previewData"
            :start-date-html="startDateHtml" :end-date-html="endDateHtml"
            :round="round" :history-entries="historyEntries" :generated="generated"
            @navigate="activeTab = $event"
            @load-entry="applyHistoryEntry"
            @connection-status="handleConnectionStatus" />
        <TabSettings v-show="activeTab === 'settings'" :db-config="dbConfig" :db-connected="dbConnected"
            @update:db-config="Object.assign(dbConfig, $event)" @save="saveDbConfig"
            @connection-status="handleConnectionStatus" />
        <TabQuery v-show="activeTab === 'query'" :db-config="dbConfig"
            v-model:start-date-html="startDateHtml"
            v-model:end-date-html="endDateHtml"
            v-model:output-dir="outputDir"
            v-model:preview-data="previewData"
            v-model:round="round"
            @navigate="activeTab = $event" />
        <TabNumberLocks v-show="activeTab === 'numberLocks'" />
        <TabReport1 v-show="activeTab === 'report1'" :db-config="dbConfig"
            :date-from="dateFrom" :date-to="dateTo"
            v-model:year="year" :month="month" :round="round"
            :output-dir="outputDir" :preview-data="previewData"
            v-model:start-reg-no="r1Form.startRegNo"
            v-model:start-running="r1Form.startRunning"
            @save-history="saveEntry"
            @navigate="activeTab = $event"
            @generated="generated.report1 = true" />
        <TabReport2 v-show="activeTab === 'report2'" :db-config="dbConfig"
            :date-from="dateFrom" :date-to="dateTo"
            v-model:year="year" :month="month" :round="round"
            :output-dir="outputDir" :preview-data="previewData"
            v-model:start-po-no="r2Form.startPoNo"
            v-model:start-purchase-no="r2Form.startPurchaseNo"
            v-model:start-reg-no="r2Form.startRegNo"
            v-model:start-running="r2Form.startRunning"
            v-model:approval-date="r2Form.approvalDate"
            @save-history="saveEntry" @carry-result="handleR2Carry"
            @navigate="activeTab = $event"
            @generated="generated.report2 = true" />
        <TabReport3 v-show="activeTab === 'report3'" :db-config="dbConfig"
            :date-from="dateFrom" :date-to="dateTo"
            v-model:year="year" :month="month" :round="round"
            :output-dir="outputDir" :preview-data="previewData"
            v-model:budget-total="r3Form.budgetTotal"
            v-model:previous-balance="r3Form.previousBalance"
            v-model:approval-date="r3Form.approvalDate"
            :r2-carry="r2Carry" @save-history="saveEntry"
            @navigate="activeTab = $event"
            @generated="generated.report3 = true" />
        <TabHistory v-show="activeTab === 'history'"
            :entries="historyEntries"
            @load-entry="applyHistoryEntry"
            @delete-entry="deleteEntry" />
    </main>

    <ToastContainer />
</div>
</template>

<style>
/* App shell only - design tokens and components live in design-system.css. */
</style>
