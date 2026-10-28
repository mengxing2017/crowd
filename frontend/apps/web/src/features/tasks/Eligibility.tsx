import { useEffect, useState } from 'react'
import { Alert, Button, Form, Input, Select } from 'antd'
import { tasks, type TaskEligibility } from './api'
import type { Run, Save } from '../platform/model'

export function Eligibility({ userId, busy, pending, run, save }: { userId: string; busy: boolean; pending: boolean; run: Run; save: Save }) {
  const [current, setCurrent] = useState<{ owner: string; data: TaskEligibility }>()
  const [form] = Form.useForm()
  async function load() {
    const data = await tasks.eligibility(userId)
    setCurrent({ owner: userId, data })
    form.setFieldsValue({ region: data.region, qualifications: data.qualifications || [], reason: '' })
  }
  useEffect(() => { setCurrent(undefined); form.resetFields(); void run(load) }, [userId])
  const available = current?.owner === userId
  return <Form name={`task-eligibility-${userId}`} form={form} layout="vertical" disabled={busy || pending || !available} onFinish={async values => {
    if (available && await save('eligibility_verify', { ...values, user_id: userId, version: current.data.version })) await run(load)
  }}>
    <Alert type="info" title="根据业务依据人工核验后录入。领取不能使用客户端自报地区或资格。每次修改保留历史。" />
    <Form.Item name="region" label="地区代码" rules={[{ required: true }]}><Input /></Form.Item>
    <Form.Item name="qualifications" label="已核验资格"><Select mode="tags" /></Form.Item>
    <Form.Item name="reason" label="核验依据" rules={[{ required: true }]}><Input.TextArea /></Form.Item>
    <Button htmlType="submit" type="primary">保存核验记录</Button>
  </Form>
}
