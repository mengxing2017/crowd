import type { Transport } from '../common/types.js'
import { createResource } from '../common/resource.js'
import { validate } from './validation.js'
import type { NotificationsRecord } from './types.js'
export function createNotifications(transport: Transport) {
 const resource = createResource(transport, validate)
 return {
    messages: (target?: string) => resource.read<NotificationsRecord[]>('messages', target),
    notificationTemplates: (target?: string) => resource.read<NotificationsRecord[]>('notification_templates', target),
    notificationTodos: (target?: string) => resource.read<NotificationsRecord[]>('notification_todos', target),
    notificationRead: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('notification_read', body, key),
    notificationConsent: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('notification_consent', body, key),
    notificationTemplate: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('notification_template', body, key),
    notificationDelivery: (body: Record<string, unknown>, key: string) => resource.command<Record<string, unknown>>('notification_delivery', body, key),
 }
}
