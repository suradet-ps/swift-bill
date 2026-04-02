<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface DbConfig {
    host: string;
    port: number;
    database: string;
    username: string;
    password: string;
}

const props = defineProps<{ dbConfig: DbConfig }>();
const emit = defineEmits<{
    (e: "update:dbConfig", val: DbConfig): void;
}>();

const status = ref<"idle" | "testing" | "success" | "error">("idle");
const message = ref("");

const isValid = computed(
    () =>
        props.dbConfig.host.trim() !== "" &&
        props.dbConfig.port > 0 &&
        props.dbConfig.database.trim() !== "" &&
        props.dbConfig.username.trim() !== "" &&
        props.dbConfig.password.trim() !== ""
);

const statusClass = computed(() => ({
    "status-success": status.value === "success",
    "status-error": status.value === "error",
    "status-info": status.value === "testing",
}));

const statusIcon = computed(() => {
    if (status.value === "success") return "✅";
    if (status.value === "error") return "❌";
    return "⏳";
});

function update(field: keyof DbConfig, value: string | number) {
    emit("update:dbConfig", { ...props.dbConfig, [field]: value });
}

async function testConnection() {
    if (!isValid.value) {
        message.value = "กรุณากรอกข้อมูลให้ครบถ้วน";
        status.value = "error";
        return;
    }
    status.value = "testing";
    message.value = "กำลังทดสอบการเชื่อมต่อ...";
    try {
        const msg = await invoke<string>("test_connection", {
            config: { ...props.dbConfig },
        });
        status.value = "success";
        message.value = msg;
    } catch (e) {
        status.value = "error";
        message.value = String(e);
    }
}
</script>

<template>
<div class="settings-wrap">
    <div class="card">
        <div class="card-title">⚙️ ตั้งค่าการเชื่อมต่อฐานข้อมูล</div>
        <div class="card-desc">เชื่อมต่อ SQL Server (INVS) ผ่าน TDS Protocol โดยตรง — ไม่ต้องติดตั้ง ODBC Driver</div>

        <div class="form-grid">
            <div class="form-group">
                <label>Host / IP Address</label>
                <input type="text" :value="dbConfig.host"
                    @input="update('host', ($event.target as HTMLInputElement).value)" placeholder="192.168.1.100" />
            </div>
            <div class="form-group">
                <label>Port</label>
                <input type="number" :value="dbConfig.port"
                    @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 1433)"
                    placeholder="1433" />
            </div>
            <div class="form-group">
                <label>Database Name</label>
                <input type="text" :value="dbConfig.database"
                    @input="update('database', ($event.target as HTMLInputElement).value)" placeholder="INVS" />
            </div>
            <div class="form-group">
                <label>Username</label>
                <input type="text" :value="dbConfig.username"
                    @input="update('username', ($event.target as HTMLInputElement).value)" placeholder="sa" />
            </div>
            <div class="form-group">
                <label>Password</label>
                <input type="password" :value="dbConfig.password"
                    @input="update('password', ($event.target as HTMLInputElement).value)" placeholder="••••••••" />
            </div>
        </div>

        <div class="actions">
            <button class="btn btn-primary btn-lg" :disabled="status === 'testing' || !isValid" @click="testConnection">
                <span v-if="status === 'testing'" class="spinner"></span>
                {{ status === "testing" ? "กำลังทดสอบ..." : "🔗 ทดสอบการเชื่อมต่อ" }}
            </button>
        </div>

        <div v-if="message" :class="['status-msg', statusClass]" style="margin-top:12px">
            <span>{{ statusIcon }}</span>
            {{ message }}
        </div>

        <div class="info-box" style="margin-top:20px">
            <strong>💡 หมายเหตุ:</strong>
            ระบบนี้อ่านข้อมูลจากตาราง <code>MS_IVO</code> และ <code>COMPANY</code> เท่านั้น
            ไม่มีการเขียนหรือแก้ไขข้อมูลในฐานข้อมูล
        </div>
    </div>

    <div class="card">
        <div class="card-title">📋 ข้อมูลที่ระบบดึงจาก INVS</div>
        <table class="data-table">
            <thead>
                <tr>
                    <th>ตาราง</th>
                    <th>คอลัมน์ที่ใช้</th>
                    <th>หมายเหตุ</th>
                </tr>
            </thead>
            <tbody>
                <tr>
                    <td><code>MS_IVO</code></td>
                    <td><code>INVOICE_NO, VENDOR_CODE, TOTAL_COST, RECEIVE_DATE</code></td>
                    <td>ใบแจ้งหนี้รับยา</td>
                </tr>
                <tr>
                    <td><code>COMPANY</code></td>
                    <td><code>COMPANY_CODE, COMPANY_NAME, KEY_WORD</code></td>
                    <td>KEY_WORD ขึ้นต้น PS = วัสดุเภสัชกรรม, อื่นๆ = ยา</td>
                </tr>
            </tbody>
        </table>
    </div>
</div>
</template>

<style scoped>
.settings-wrap {
    max-width: 800px;
}

.actions {
    margin-top: 20px;
}

code {
    font-family: monospace;
    font-size: 12px;
    background: #f3f4f6;
    padding: 1px 5px;
    border-radius: 3px;
    color: #374151;
}
</style>
