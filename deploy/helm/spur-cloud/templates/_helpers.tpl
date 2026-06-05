{{/*
Expand the name of the chart.
*/}}
{{- define "spur-cloud.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{/*
Fully qualified app name.
*/}}
{{- define "spur-cloud.fullname" -}}
{{- if .Values.fullnameOverride -}}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- $name := default .Chart.Name .Values.nameOverride -}}
{{- if contains $name .Release.Name -}}
{{- .Release.Name | trunc 63 | trimSuffix "-" -}}
{{- else -}}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" -}}
{{- end -}}
{{- end -}}
{{- end -}}

{{- define "spur-cloud.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" -}}
{{- end -}}

{{- define "spur-cloud.labels" -}}
helm.sh/chart: {{ include "spur-cloud.chart" . }}
app.kubernetes.io/name: {{ include "spur-cloud.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end -}}

{{- define "spur-cloud.api.fullname" -}}
{{ include "spur-cloud.fullname" . }}-api
{{- end -}}

{{- define "spur-cloud.frontend.fullname" -}}
{{ include "spur-cloud.fullname" . }}-frontend
{{- end -}}

{{- define "spur-cloud.postgres.fullname" -}}
{{ include "spur-cloud.fullname" . }}-postgres
{{- end -}}

{{- define "spur-cloud.serviceAccountName" -}}
{{- if .Values.serviceAccount.create -}}
{{- default (include "spur-cloud.fullname" .) .Values.serviceAccount.name -}}
{{- else -}}
{{- default "default" .Values.serviceAccount.name -}}
{{- end -}}
{{- end -}}

{{- define "spur-cloud.secretName" -}}
{{- if .Values.secrets.existingSecret -}}
{{- .Values.secrets.existingSecret -}}
{{- else -}}
{{ include "spur-cloud.fullname" . }}-secrets
{{- end -}}
{{- end -}}

{{- define "spur-cloud.api.image" -}}
{{- $tag := default .Chart.AppVersion .Values.api.image.tag -}}
{{- printf "%s:%s" .Values.api.image.repository $tag -}}
{{- end -}}

{{- define "spur-cloud.frontend.image" -}}
{{- $tag := default .Chart.AppVersion .Values.frontend.image.tag -}}
{{- printf "%s:%s" .Values.frontend.image.repository $tag -}}
{{- end -}}

{{/*
Resolve the database URL: explicit override wins, otherwise build from
in-cluster Postgres service if enabled.
*/}}
{{- define "spur-cloud.databaseUrl" -}}
{{- if .Values.database.url -}}
{{- .Values.database.url -}}
{{- else if .Values.postgres.enabled -}}
{{- if not .Values.secrets.dbPassword -}}
{{- fail "secrets.dbPassword must be set when postgres.enabled is true and secrets.existingSecret is unused" -}}
{{- end -}}
{{- printf "postgresql://%s:%s@%s:5432/%s" .Values.postgres.user .Values.secrets.dbPassword (include "spur-cloud.postgres.fullname" .) .Values.postgres.database -}}
{{- else -}}
{{- fail "Either postgres.enabled must be true or database.url must be set" -}}
{{- end -}}
{{- end -}}
