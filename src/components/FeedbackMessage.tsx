import type { ReactNode } from "react";

type FeedbackVariant = "error" | "success" | "info" | "warning" | "status";

const accessibilityByVariant: Record<FeedbackVariant, { role: "alert" | "status"; ariaLive: "assertive" | "polite" }> = {
  error: { role: "alert", ariaLive: "assertive" },
  warning: { role: "alert", ariaLive: "polite" },
  success: { role: "status", ariaLive: "polite" },
  info: { role: "status", ariaLive: "polite" },
  status: { role: "status", ariaLive: "polite" },
};

export function FeedbackMessage({
  variant,
  children,
  className,
}: {
  variant: FeedbackVariant;
  children: ReactNode;
  className?: string;
}) {
  const accessibility = accessibilityByVariant[variant];

  return (
    <div
      className={["feedback-message", `feedback-message--${variant}`, className].filter(Boolean).join(" ")}
      role={accessibility.role}
      aria-live={accessibility.ariaLive}
    >
      {children}
    </div>
  );
}
