import { createNotifications } from '@crowd/api-contract/notifications'
import { transport } from '../../session/api'
export const notifications = createNotifications(transport)
export type * from '@crowd/api-contract/notifications'
export const notificationsCommands = {
 notification_read: notifications.notificationRead,
 notification_consent: notifications.notificationConsent,
 notification_template: notifications.notificationTemplate,
 notification_delivery: notifications.notificationDelivery,
}
