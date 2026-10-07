<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
    CheckCircle,
    Database,
    Eye,
    EyeOff,
    Lock,
    Plug,
    Save,
    Server,
    ShieldCheck,
    Table2,
    User,
    XCircle,
} from "lucide-vue-next";
import type { DbConfig } from "../lib/types";

const props = defineProps<{
    dbConfig: DbConfig;
    dbConnected: boolean | null;
}>();

const emit = defineEmits<{
    (e: "update:dbConfig", val: DbConfig): void;
    (e: "save"): void;
    (e: "connectionStatus", connected: boolean): void;
}>();

const status = ref<"idle" | "testing" | "success" | "error">("idle");
const message = ref("");
const saveStatus = ref<"idle" | "saved">("idle");
const showPassword = ref(false);

const isValid = computed(
    () =>
        props.dbConfig.host.trim() !== "" &&
        props.dbConfig.port > 0 &&
        props.dbConfig.database.trim() !== "" &&
        props.dbConfig.username.trim() !== "" &&
        props.dbConfig.password.trim() !== ""
);

function update(field: keyof DbConfig, value: string | number) {
    emit("update:dbConfig", { ...props.dbConfig, [field]: value });
}

function saveConfig() {
    emit("save");
    saveStatus.value = "saved";
    setTimeout(() => {
        saveStatus.value = "idle";
    }, 2500);
}

async function testConnection() {
    if (!isValid.value) {
        message.value = "กรุณากรอกข้อมูลให้ครบถ้วนก่อนทดสอบ";
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
        emit("connectionStatus", true);
    } catch (e) {
        status.value = "error";
        message.value = String(e);
        emit("connectionStatus", false);
    }
}
</script>

<template>
<div class="settings-wrap">
    <div class="page-header">
        <div class="page-header-text">
            <h2 class="page-title">ตั้งค่าฐานข้อมูล</h2>
        </div>
        <div class="page-actions">
            <span class="badge" :class="dbConnected === true ? 'badge-success' : dbConnected === false ? 'badge-danger' : 'badge-neutral'">
                <span class="conn-dot" :class="dbConnected === true ? 'ok' : dbConnected === false ? 'fail' : 'unknown'"></span>
                {{ dbConnected === true ? "เชื่อมต่อแล้ว" : dbConnected === false ? "เชื่อมต่อไม่สำเร็จ" : "ยังไม่ได้ทดสอบ" }}
            </span>
        </div>
    </div>

    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><Server :size="16" /> การเชื่อมต่อ SQL Server</div>
            </div>
        </div>

        <div class="section-label"><Database :size="13" /> เซิร์ฟเวอร์</div>
        <div class="form-grid">
            <div class="form-group">
                <label for="db-host">โฮสต์ / IP ของเซิร์ฟเวอร์</label>
                <input id="db-host" type="text" :value="dbConfig.host"
                    @input="update('host', ($event.target as HTMLInputElement).value)"
                    placeholder="เช่น 192.168.1.100" autocapitalize="none" autocorrect="off"
                    spellcheck="false" />
            </div>
            <div class="form-group">
                <label for="db-port">พอร์ต</label>
                <input id="db-port" type="number" :value="dbConfig.port"
                    @input="update('port', parseInt(($event.target as HTMLInputElement).value) || 1433)"
                    placeholder="1433" />
            </div>
            <div class="form-group">
                <label for="db-name">ชื่อฐานข้อมูล</label>
                <input id="db-name" type="text" :value="dbConfig.database"
                    @input="update('database', ($event.target as HTMLInputElement).value)"
                    placeholder="INVS" autocapitalize="none" autocorrect="off" spellcheck="false" />
            </div>
        </div>

        <div class="section-label section-spaced"><User :size="13" /> ผู้ใช้งาน</div>
        <div class="form-grid-2">
            <div class="form-group">
                <label for="db-user">ชื่อผู้ใช้</label>
                <input id="db-user" type="text" :value="dbConfig.username"
                    @input="update('username', ($event.target as HTMLInputElement).value)"
                    placeholder="เช่น sa" autocapitalize="none" autocorrect="off" spellcheck="false" />
            </div>
            <div class="form-group">
                <label for="db-pass">รหัสผ่าน</label>
                <div class="input-group">
                    <input id="db-pass" :type="showPassword ? 'text' : 'password'"
                        :value="dbConfig.password"
                        @input="update('password', ($event.target as HTMLInputElement).value)"
                        placeholder="••••••••" />
                    <button type="button" class="btn btn-secondary btn-icon"
                        :aria-label="showPassword ? 'ซ่อนรหัสผ่าน' : 'แสดงรหัสผ่าน'"
                        @click="showPassword = !showPassword">
                        <EyeOff v-if="showPassword" :size="15" />
                        <Eye v-else :size="15" />
                    </button>
                </div>
            </div>
        </div>

        <div class="info-box section-spaced">
            <ShieldCheck :size="15" />
            <span>รหัสผ่านถูกเข้ารหัสและเก็บใน Keychain ของเครื่อง</span>
        </div>

        <div class="actions actions-row">
            <button class="btn btn-primary" :disabled="status === 'testing' || !isValid"
                @click="testConnection">
                <span v-if="status === 'testing'" class="spinner"></span>
                <Plug v-else :size="15" />
                {{ status === "testing" ? "กำลังทดสอบ..." : "ทดสอบการเชื่อมต่อ" }}
            </button>
            <button class="btn btn-secondary" :disabled="!isValid" @click="saveConfig">
                <CheckCircle v-if="saveStatus === 'saved'" :size="15" />
                <Save v-else :size="15" />
                {{ saveStatus === "saved" ? "บันทึกแล้ว" : "บันทึกการตั้งค่า" }}
            </button>
        </div>

        <div v-if="message"
            :class="['status-msg', 'status-stack',
                status === 'success' ? 'status-success' : status === 'error' ? 'status-error' : 'status-info']">
            <CheckCircle v-if="status === 'success'" :size="15" />
            <XCircle v-else-if="status === 'error'" :size="15" />
            <span v-else class="spinner"></span>
            {{ message }}
        </div>
    </div>

    <div class="card">
        <div class="card-head">
            <div>
                <div class="card-title"><Table2 :size="16" /> ข้อมูลที่ระบบดึงจาก INVS</div>
            </div>
        </div>
        <div class="table-wrap">
            <table class="data-table">
                <thead>
                    <tr>
                        <th>ตาราง</th>
                        <th>คอลัมน์ที่ใช้</th>
                        <th class="text-center">สิทธิ์</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td><code>MS_IVO</code></td>
                        <td><code>INVOICE_NO, VENDOR_CODE, TOTAL_COST, RECEIVE_DATE</code></td>
                        <td class="text-center"><span class="badge badge-success"><Lock :size="11" /> อ่านเท่านั้น</span></td>
                    </tr>
                    <tr>
                        <td><code>COMPANY</code></td>
                        <td><code>COMPANY_CODE, COMPANY_NAME, KEY_WORD</code></td>
                        <td class="text-center"><span class="badge badge-success"><Lock :size="11" /> อ่านเท่านั้น</span></td>
                    </tr>
                </tbody>
            </table>
        </div>
    </div>
</div>
</template>
