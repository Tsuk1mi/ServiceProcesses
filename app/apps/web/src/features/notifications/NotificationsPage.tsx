import { Link } from "react-router-dom";
import { PageHeader } from "@/shared/components/PageHeader";
import { EmptyState } from "@/shared/components/EmptyState";
import { useAppStore } from "@/app/store";

export function NotificationsPage() {
  const notifications = useAppStore((state) => state.notifications);
  return (
    <>
      <PageHeader title="Уведомления" />
      <div className="toolbar"><button>Все</button><button>Непрочитанные</button><button>SLA</button></div>
      <section className="panel notification-list">
        {notifications.length ? (
          notifications.map((item) => (
            <Link key={item.id} to={item.href} className={item.read ? "" : "unread"}>
              <strong>{item.title}</strong>
              <span>{item.body}</span>
            </Link>
          ))
        ) : (
          <EmptyState title="Уведомлений пока нет" description="После появления заявок и SLA-событий здесь появится активность системы." />
        )}
      </section>
    </>
  );
}
