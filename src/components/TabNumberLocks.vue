<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../composables/useToast";
import { Lock, PlusCircle, Trash2, CalendarDays, FileLock2 } from "lucide-vue-next";

interface NumberLockEntry {
    id: string;
    fiscal_year: number;
    request_no: number;
    report_no: number;
    purchase_no: number;
    reason: string;
    note: string;
    created_at: string;
}

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
const selectedFiscalYear = ref<number>(currentFiscalYear);

const previewRows = computed(() =>
    Array.from({ length: Math.max(form.count, 0) }, (_, idx) => {
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
        await loadEntries();
        toast.success("ลบเลขล็อกสำเร็จ", "ระบบนำเลขชุดนี้ออกจากรายการล็อกแล้ว");
    } catch (e) {
        toast.error("ลบเลขล็อกล้มเหลว", String(e));
    } finally {
        deletingId.value = null;
    }
}

function formatDateTime(iso: string): string {
    try {
        return new Date(iso).toLocaleString("th-TH", {
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    } catch {
        return iso;
    }
}

onMounted(loadEntries);
</script>

<template>
<div class="lock-wrap">
    <div class="page-header">
        <h2 class="page-title">ล็อกเลข</h2>
        <p class="page-desc">กันเลขชุดที่ไม่ต้องการใช้งาน เพื่อให้ระบบข้ามก่อนจัดสรรเลขขอซื้อ รายงาน และใบสั่งซื้อ</p>
    </div>

    <div class="card">
        <div class="card-title">
            <Lock :size="17" /> สร้างเลขล็อก
        </div>
        <div class="card-desc">ล็อกเป็นชุดของบิล 1 ชุด โดยรายงานจะคำนวณจากเลขขอซื้อ + 1 ให้อัตโนมัติ</div>

        <div class="form-grid">
            <div class="form-group">
                <label>ปีงบประมาณ</label>
                <input v-model.number="form.fiscalYear" type="number" min="2500" />
            </div>
            <div class="form-group">
                <label>เลขขอซื้อเริ่มต้น</label>
                <input v-model.number="form.startRequestNo" type="number" min="1" />
            </div>
            <div class="form-group">
                <label>เลขใบสั่งซื้อเริ่มต้น</label>
                <input v-model.number="form.startPurchaseNo" type="number" min="1" />
            </div>
            <div class="form-group">
                <label>จำนวนชุดที่ต้องการล็อก</label>
                <input v-model.number="form.count" type="number" min="1" />
            </div>
            <div class="form-group">
                <label>เหตุผล</label>
                <input v-model="form.reason" type="text" placeholder="เช่น กันเลขไว้ใช้หน้างาน" />
            </div>
            <div class="form-group">
                <label>หมายเหตุ</label>
                <input v-model="form.note" type="text" placeholder="ไม่บังคับ" />
            </div>
        </div>

        <div class="actions">
            <button class="btn btn-primary btn-lg" :disabled="!canSave || saving" @click="createLocks">
                <span v-if="saving" class="spinner"></span>
                <PlusCircle v-else :size="16" />
                {{ saving ? "กำลังบันทึก..." : "บันทึกเลขล็อก" }}
            </button>
        </div>
    </div>

    <div class="card">
        <div class="card-title">
            <FileLock2 :size="17" /> ตัวอย่างชุดที่จะถูกล็อก
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
    </div>

    <div class="card">
        <div class="card-title">
            <CalendarDays :size="17" /> รายการเลขที่ล็อกไว้
        </div>
        <div class="card-desc">ระบบตรวจรายการนี้ทุกครั้งก่อนโหลดเลขจากประวัติ, แสดงตัวอย่าง, และส่งออกรายงานสรุปรับยา</div>

        <div class="filter-row">
            <div class="form-group fiscal-filter">
                <label>กรองตามปีงบประมาณ</label>
                <input v-model.number="selectedFiscalYear" type="number" min="2500" />
            </div>
            <div class="summary-chip">
                {{ filteredEntries.length }} ชุดในปี {{ selectedFiscalYear }}
            </div>
        </div>

        <div v-if="loading" class="status-msg status-info status-stack">
            <span class="spinner"></span> กำลังโหลดข้อมูล...
        </div>

        <div v-else-if="filteredEntries.length === 0" class="empty-state">
            ยังไม่มีเลขล็อกสำหรับปีงบประมาณนี้
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
                        <td>{{ entry.note || "—" }}</td>
                        <td>{{ formatDateTime(entry.created_at) }}</td>
                        <td class="text-center">
                            <button class="btn btn-danger btn-sm" :disabled="deletingId === entry.id"
                                @click="removeEntry(entry.id)">
                                <span v-if="deletingId === entry.id" class="spinner"></span>
                                <Trash2 v-else :size="14" />
                                ลบ
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
.lock-wrap {
    display: flex;
    flex-direction: column;
    gap: 18px;
}

.filter-row {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 14px;
    margin-bottom: 14px;
}

.fiscal-filter {
    min-width: 220px;
    margin-bottom: 0;
}

.summary-chip {
    display: inline-flex;
    align-items: center;
    padding: 9px 12px;
    border-radius: 9999px;
    background: var(--c-primary-light);
    color: var(--c-primary);
    font-size: 12px;
    font-weight: 600;
    box-shadow: rgba(200, 16, 46, 0.12) 0px 0px 0px 1px;
}

.empty-state {
    text-align: center;
    padding: 26px 18px;
    border-radius: var(--radius-lg);
    background: var(--c-surface-raised);
    color: var(--c-text-light);
    box-shadow: var(--shadow-ring);
}
</style>
