<script setup lang="ts">
import { ref } from "vue";
import { cleanSourceLabel, formatBuddhistDate, formatDateTime, formatMoney } from "../lib/format";
import type { RoundHistoryEntry } from "../lib/types";
import {
    ArrowRight,
    Banknote,
    CalendarDays,
    Check,
    Clock,
    Download,
    FolderOpen,
    Package,
    Trash2,
    X,
} from "lucide-vue-next";

defineProps<{
    entries: RoundHistoryEntry[];
}>();

const emit = defineEmits<{
    (e: "loadEntry", entry: RoundHistoryEntry): void;
    (e: "deleteEntry", id: string): void;
}>();

const confirmingId = ref<string | null>(null);

function doDelete(id: string) {
    emit("deleteEntry", id);
    confirmingId.value = null;
}
</script>

<template>
<div class="history-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">ประวัติรอบ</h2>
        </div>
    </div>

    <!-- Empty state -->
    <div v-if="entries.length === 0" class="card">
        <div class="empty-state">
            <div class="empty-icon"><FolderOpen :size="44" stroke-width="1.5" /></div>
            <div class="empty-title">ยังไม่มีประวัติรอบ</div>
            <p class="empty-desc">บันทึกรอบจากหน้าผลรายงานเพื่อใช้ต่อรอบถัดไป</p>
        </div>
    </div>

    <!-- Entry list -->
    <div v-else class="entries">
        <div v-for="entry in entries" :key="entry.id" class="entry-card">
            <div class="entry-header">
                <div class="entry-title-row">
                    <span class="entry-label">{{ entry.label }}</span>
                    <span v-if="entry.source_tab" class="badge badge-neutral">{{ cleanSourceLabel(entry.source_tab) }}</span>
                </div>
                <div class="entry-meta">
                    <span class="meta-chip">
                        <CalendarDays :size="12" />
                        {{ formatBuddhistDate(entry.date_from) }} - {{ formatBuddhistDate(entry.date_to) }}
                    </span>
                    <span class="meta-chip">
                        <Package :size="12" /> {{ entry.invoice_count }} บิล
                    </span>
                    <span class="meta-chip money">
                        <Banknote :size="12" /> {{ formatMoney(entry.total_amount) }} บาท
                    </span>
                    <span class="meta-chip muted">
                        <Clock :size="12" /> {{ formatDateTime(entry.created_at) }}
                    </span>
                </div>
            </div>

            <div class="carry-section">
                <div class="carry-title">
                    <ArrowRight :size="14" /> ค่าสำหรับรอบถัดไป (รอบ {{ entry.round + 1 }})
                </div>
                <div class="carry-values">
                    <div class="cv-item">
                        <span class="cv-label">เลขทะเบียนคุม</span>
                        <span class="cv-val reg">{{ entry.next_reg_no || "-" }}</span>
                    </div>
                    <div class="cv-item">
                        <span class="cv-label">ลำดับในสมุด</span>
                        <span class="cv-val">{{ entry.next_running }}</span>
                    </div>
                    <div class="cv-item">
                        <span class="cv-label">เลขขอซื้อ/PO</span>
                        <span class="cv-val">{{ entry.next_po_no || "-" }}</span>
                    </div>
                    <div class="cv-item">
                        <span class="cv-label">ยอดงบคงเหลือ</span>
                        <span class="cv-val money">
                            {{ entry.remaining_balance > 0 ? formatMoney(entry.remaining_balance) : "-" }}
                        </span>
                    </div>
                    <div v-if="entry.budget_total > 0" class="cv-item">
                        <span class="cv-label">งบประมาณรวม</span>
                        <span class="cv-val">{{ formatMoney(entry.budget_total) }}</span>
                    </div>
                </div>
            </div>

            <div class="entry-actions">
                <button class="btn btn-primary" @click="emit('loadEntry', entry)">
                    <Download :size="15" /> โหลดค่านี้ไปใช้รอบถัดไป
                </button>
                <template v-if="confirmingId === entry.id">
                    <span class="confirm-text">ยืนยันลบรอบนี้?</span>
                    <button class="btn btn-danger" @click="doDelete(entry.id)">
                        <Check :size="15" /> ยืนยันลบ
                    </button>
                    <button class="btn btn-secondary" @click="confirmingId = null">
                        <X :size="15" /> ยกเลิก
                    </button>
                </template>
                <button v-else class="btn btn-ghost" @click="confirmingId = entry.id">
                    <Trash2 :size="15" /> ลบ
                </button>
            </div>
        </div>
    </div>
</div>
</template>

<style scoped>
</style>
