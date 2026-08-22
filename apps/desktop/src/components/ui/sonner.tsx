import { useTheme } from "next-themes"
import { Toaster as Sonner } from "sonner"

type ToasterProps = React.ComponentProps<typeof Sonner>

const Toaster = ({ ...props }: ToasterProps) => {
  const { theme = "system" } = useTheme()

  return (
    <Sonner
      theme={theme as ToasterProps["theme"]}
      className="toaster group"
      toastOptions={{
        classNames: {
          toast:
            "group toast group-[.toaster]:bg-popover group-[.toaster]:text-popover-foreground group-[.toaster]:border-border group-[.toaster]:shadow-lg dark:group-[.toaster]:bg-card dark:group-[.toaster]:text-popover-foreground dark:group-[.toaster]:border-border/10",
          description: "group-[.toast]:text-muted-foreground dark:group-[.toast]:text-muted-foreground",
          actionButton:
            "group-[.toast]:bg-secondary group-[.toast]:text-popover-foreground dark:group-[.toast]:bg-border dark:group-[.toast]:text-muted-foreground",
          cancelButton:
            "group-[.toast]:bg-secondary group-[.toast]:text-muted-foreground dark:group-[.toast]:bg-secondary dark:group-[.toast]:text-muted-foreground",
        },
      }}
      {...props}
    />
  )
}

export { Toaster }




