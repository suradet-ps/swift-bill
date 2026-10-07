<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { formatDateTime } from "../lib/format";
import type { NumberLockEntry } from "../lib/types";
import {
    CalendarDays,
    Check,
    FileLock2,
    Lock,
    PlusCircle,
    Trash2,
    X,
} from "lucide-vue-next";

const toast = useToast();

const currentFiscalYear = new Date().getFullYear() + 543;

const form = reactive({
    fiscalYear: currentFiscalYear,
    startRequestNo: 1,
    startPurchaseNo: 1,
    count: 1,
    reason: "",
    note: "",
});

const entries = ref<NumberLockEntry[]>([]);
const loading = ref(false);
const saving = ref(false);
const deletingId = ref<string | null>(null);
const confirmingId = ref<string | null>(null);
const selectedFiscalYear = ref<number>(currentFiscalYear);

const previewRows = computed(() =>
    Array.from({ length: Math.max(Math.min(form.count, 20), 0) }, (_, idx) => {
        const requestNo = form.startRequestNo + idx * 2;
        return {
            request_no: requestNo,
            report_no: requestNo + 1,
            purchase_no: form.startPurchaseNo + idx,
        };
    })
);

const filteredEntries = computed(() =>
    entries.value.filter((entry) => entry.fiscal_year === selectedFiscalYear.value)
);

const canSave = computed(
    () =>
        form.fiscalYear > 0 &&
        form.startRequestNo > 0 &&
        form.startPurchaseNo > 0 &&
        form.count > 0 &&
        form.reason.trim() !== ""
);

async function loadEntries() {
    loading.value = true;
    try {
        entries.value = await invoke<NumberLockEntry[]>("load_number_locks");
    } catch (e) {
        toast.error("โหลดเลขล็อกล้มเหลว", String(e));
    } finally {
        loading.value = false;
    }
}

async function createLocks() {
    if (!canSave.value) return;
    saving.value = true;
    try {
        await invoke("create_number_locks", {
            params: {
                fiscal_year: form.fiscalYear,
                start_request_no: form.startRequestNo,
                start_purchase_no: form.startPurchaseNo,
                count: form.count,
                reason: form.reason.trim(),
                note: form.note.trim(),
            },
        });
        selectedFiscalYear.value = form.fiscalYear;
        form.reason = "";
        form.note = "";
        form.count = 1;
        await loadEntries();
        toast.success("บันทึกเลขล็อกสำเร็จ", "ระบบจะข้ามเลขชุดนี้ทุกครั้งก่อนจัดสรรเลข");
    } catch (e) {
        toast.error("บันทึกเลขล็อกล้มเหลว", String(e));
    } finally {
        saving.value = false;
    }
}

async function removeEntry(id: string) {
    deletingId.value = id;
    try {
        await invoke("delete_number_lock", { id });
        confirmingId.value = null;
        await loadEntries();
        toast.success("ลบเลขล็อกสำเร็จ", "ระบบนำเลขชุดนี้ออกจากรายการล็อกแล้ว");
    } catch (e) {
        toast.error("ลบเลขล็อกล้มเหลว", String(e));
    } finally {
        deletingId.value = null;
    }
}

onMounted(loadEntries);
</script>

<template>
<div class="lock-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">ล็อกเลข</h2>
            <p class="page-desc">กันเลขชุดที่ไม่ใช้งาน ระบบจะข้ามให้อัตโนมัติ</p>
        </div>
    </div>

    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><Lock :size="16" /> สร้างเลขล็อกชุดใหม่</div>
            </div>
        </div>

        <div class="form-grid">
            <div class="form-group">
                <label for="lock-year">ปีงบประมาณ</label>
                <input id="lock-year" v-model.number="form.fiscalYear" type="number" min="2500" />
            </div>
            <div class="form-group">
                <label for="lock-request">เลขขอซื้อเริ่มต้น</label>
                <input id="lock-request" v-model.number="form.startRequestNo" type="number" min="1" />
            </div>
            <div class="form-group">
                <label for="lock-purchase">เลขใบสั่งซื้อเริ่มต้น</label>
                <input id="lock-purchase" v-model.number="form.startPurchaseNo" type="number" min="1" />
            </div>
            <div class="form-group">
                <label for="lock-count">จำนวนชุดที่ต้องการล็อก</label>
                <input id="lock-count" v-model.number="form.count" type="number" min="1" />
            </div>
            <div class="form-group">
                <label for="lock-reason">เหตุผล (จำเป็น)</label>
                <input id="lock-reason" v-model="form.reason" type="text"
                    placeholder="เช่น กันเลขไว้ใช้หน้างาน" />
            </div>
            <div class="form-group">
                <label for="lock-note">หมายเหตุ</label>
                <input id="lock-note" v-model="form.note" type="text" placeholder="ไม่บังคับ" />
            </div>
        </div>

        <div class="actions actions-row">
            <button class="btn btn-primary" :disabled="!canSave || saving" @click="createLocks">
                <span v-if="saving" class="spinner"></span>
                <PlusCircle v-else :size="15" />
                {{ saving ? "กำลังบันทึก..." : "บันทึกเลขล็อก" }}
            </button>
        </div>
    </div>

    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><FileLock2 :size="16" /> ตัวอย่างชุดที่จะล็อก</div>
            </div>
        </div>
        <div class="table-wrap">
            <table class="data-table">
                <thead>
                    <tr>
                        <th class="text-center">#</th>
                        <th class="text-center">เลขขอซื้อ</th>
                        <th class="text-center">รายงาน</th>
                        <th class="text-center">ใบสั่งซื้อ</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="(row, idx) in previewRows" :key="`${row.request_no}-${row.purchase_no}`">
                        <td class="text-center num">{{ idx + 1 }}</td>
                        <td class="text-center">{{ row.request_no }}</td>
                        <td class="text-center">{{ row.report_no }}</td>
                        <td class="text-center">{{ row.purchase_no }}</td>
                    </tr>
                </tbody>
            </table>
        </div>
        <p v-if="form.count > previewRows.length" class="preview-note">
            แสดง {{ previewRows.length }} จาก {{ form.count }} ชุด ส่วนที่เหลือระบบจะสร้างให้ครบเมื่อบันทึก
        </p>
    </div>

    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><CalendarDays :size="16" /> รายการเลขที่ล็อกไว้</div>
            </div>
        </div>

        <div class="toolbar">
            <div class="form-group fiscal-filter">
                <label for="lock-filter">กรองตามปีงบประมาณ</label>
                <input id="lock-filter" v-model.number="selectedFiscalYear" type="number" min="2500" />
            </div>
            <span class="badge badge-neutral">{{ filteredEntries.length }} ชุดในปี {{ selectedFiscalYear }}</span>
        </div>

        <div v-if="loading" class="status-msg status-info">
            <span class="spinner"></span> กำลังโหลดข้อมูล...
        </div>

        <div v-else-if="filteredEntries.length === 0" class="empty-state compact">
            <div class="empty-icon"><FileLock2 :size="30" stroke-width="1.5" /></div>
            <div class="empty-title">ยังไม่มีเลขล็อกสำหรับปีนี้</div>
        </div>

        <div v-else class="table-wrap">
            <table class="data-table">
                <thead>
                    <tr>
                        <th class="text-center">เลขขอซื้อ</th>
                        <th class="text-center">รายงาน</th>
                        <th class="text-center">ใบสั่งซื้อ</th>
                        <th>เหตุผล</th>
                        <th>หมายเหตุ</th>
                        <th>บันทึกเมื่อ</th>
                        <th class="text-center">จัดการ</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="entry in filteredEntries" :key="entry.id">
                        <td class="text-center">{{ entry.request_no }}</td>
                        <td class="text-center">{{ entry.report_no }}</td>
                        <td class="text-center">{{ entry.purchase_no }}</td>
                        <td>{{ entry.reason }}</td>
                        <td>{{ entry.note || "-" }}</td>
                        <td class="muted">{{ formatDateTime(entry.created_at) }}</td>
                        <td class="text-center">
                            <template v-if="confirmingId === entry.id">
                                <span class="confirm-inline">
                                    <button class="btn btn-danger btn-sm" :disabled="deletingId === entry.id"
                                        @click="removeEntry(entry.id)">
                                        <span v-if="deletingId === entry.id" class="spinner"></span>
                                        <Check v-else :size="13" /> ยืนยัน
                                    </button>
                                    <button class="btn btn-secondary btn-sm" @click="confirmingId = null">
                                        <X :size="13" /> ยกเลิก
                                    </button>
                                </span>
                            </template>
                            <button v-else class="btn btn-danger btn-sm" @click="confirmingId = entry.id">
                                <Trash2 :size="13" /> ลบ
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</div>
</template>

<style scoped>
.fiscal-filter {
    max-width: 220px;
    margin-bottom: 0;
}

.confirm-inline {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    white-space: nowrap;
}

.preview-note {
    margin-top: var(--sp-2);
    font-size: var(--fs-xs);
    color: var(--c-text-light);
}
</style>
