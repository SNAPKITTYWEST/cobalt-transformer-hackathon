<#
.SYNOPSIS
  Collect COBOL sources and transformer projects from disk into one git repo.

.DESCRIPTION
  Scans -Roots, copies every COBOL file (by extension) and every file or
  folder whose name matches -TransformerPattern into -Dest, mirroring each
  original absolute path so names never collide. Every copy is SHA-256
  verified, listed in MANIFEST.csv, and committed to a git repo in -Dest.
  Nothing at the source locations is modified.

  The param block and the four helpers (New-NameSet, Test-ReparsePoint,
  Test-UnderDest, Test-TransformerName) were written to complete a script
  whose pasted copy started at Get-RepoPath; everything from Get-RepoPath
  down is unchanged.

.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File .\collect-cobol-transformer.ps1 -DryRun
  Lists what would be collected; copies nothing.
#>
[CmdletBinding()]
param(
    # Folders to scan. Default: every fixed drive.
    [string[]] $Roots = @([System.IO.DriveInfo]::GetDrives() |
        Where-Object { $_.DriveType -eq 'Fixed' -and $_.IsReady } |
        ForEach-Object { $_.RootDirectory.FullName }),

    # Where the new repo is built.
    [string] $Dest = (Join-Path $HOME 'Desktop\cobol-transformer-repo'),

    # File extensions collected as COBOL sources (with the dot, any case).
    [string[]] $Extensions = @('.cbl', '.cob', '.cobol', '.cpy', '.cbk'),

    # Folder names never descended into, anywhere in the tree: VCS and build
    # output, package and model caches, and Windows system folders.
    [string[]] $SkipDirNames = @(
        '.git', 'node_modules', '.venv', 'venv', '__pycache__', 'site-packages', 'dist-packages',
        'target', '.cargo', '.rustup', '.cache', '.conda', 'anaconda3', 'miniconda3',
        'AppData', '.claude', 'Windows', 'Program Files', 'Program Files (x86)', 'ProgramData',
        '$Recycle.Bin', 'System Volume Information'
    ),

    # Regex (case-insensitive) a file or folder name must match to count as
    # part of a transformer project. Matching folders are copied whole.
    [string] $TransformerPattern = 'transformer',

    # List what would be collected without copying anything.
    [switch] $DryRun,

    # Copy each distinct file content once; later copies are recorded in
    # MANIFEST.csv as duplicate-of:<first copy>.
    [switch] $Dedupe
)

# Case-insensitive string set (Windows paths and extensions ignore case).
function New-NameSet([string[]] $names) {
    $set = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
    foreach ($n in @($names)) { if ($n) { [void]$set.Add($n) } }
    return ,$set   # the comma stops PowerShell unrolling the set into its items
}

# Symlinks and junctions are skipped (they cause loops and double counting);
# so is anything whose attributes cannot be read.
function Test-ReparsePoint([string] $path) {
    try {
        $attrs = [System.IO.File]::GetAttributes($path)
        return [bool]($attrs -band [System.IO.FileAttributes]::ReparsePoint)
    } catch { return $true }
}

# True for the destination repo and anything inside it, so a rerun never
# collects its own output.
function Test-UnderDest([string] $path) {
    $p = $path.TrimEnd('\')
    return ($p.Equals($DestFull, [System.StringComparison]::OrdinalIgnoreCase) -or $p.StartsWith($DestFull + '\', [System.StringComparison]::OrdinalIgnoreCase))
}

function Test-TransformerName([string] $name) {
    return ($name -match $TransformerPattern)
}

# Mirror the original absolute path under a bucket so names can never collide.
function Get-RepoPath([string] $source, [string] $bucket) {
    $full = [System.IO.Path]::GetFullPath($source)
    $root = [System.IO.Path]::GetPathRoot($full)
    $tag  = ($root.TrimEnd('\') -replace '[\\:]', '_').Trim('_')
    if (-not $tag) { $tag = 'root' }
    return [System.IO.Path]::Combine($DestFull, $bucket, $tag, $full.Substring($root.Length))
}

function Add-Job([string] $src, [string] $kind, [string] $bucket) {
    if ($seenFiles.Add($src)) {
        $jobs.Add([pscustomobject]@{ Kind = $kind; Source = $src; Target = (Get-RepoPath $src $bucket) })
    }
}

# Copy a whole transformer project (minus .git / node_modules / venvs).
function Add-TransformerTree([string] $dir) {
    $pending = [System.Collections.Generic.Stack[string]]::new()
    $pending.Push($dir)
    while ($pending.Count -gt 0) {
        $d = $pending.Pop()
        try {
            foreach ($f in [System.IO.Directory]::GetFiles($d)) { Add-Job $f 'transformer' 'transformer' }
            foreach ($s in [System.IO.Directory]::GetDirectories($d)) {
                if ($skipSet.Contains([System.IO.Path]::GetFileName($s))) { continue }
                if (Test-ReparsePoint $s) { continue }
                $pending.Push($s)
            }
        } catch { $script:unreadable++ }
    }
}

# ---- 1. discover -------------------------------------------------------------
$DestFull   = [System.IO.Path]::GetFullPath($Dest).TrimEnd('\')
$extSet     = New-NameSet $Extensions
$skipSet    = New-NameSet $SkipDirNames
$seenFiles  = New-NameSet @()
$seenDirs   = New-NameSet @()
$jobs       = [System.Collections.Generic.List[object]]::new()
$stack      = [System.Collections.Generic.Stack[string]]::new()
$unreadable = 0

foreach ($r in $Roots) {
    if (Test-Path -LiteralPath $r) { $stack.Push([System.IO.Path]::GetFullPath($r)) }
    else { Write-Warning "Root not found, skipped: $r" }
}
Write-Host "Scanning: $($Roots -join ', ')  (this can take a while on a full drive)"

while ($stack.Count -gt 0) {
    $dir = $stack.Pop()
    if (-not $seenDirs.Add($dir) -or (Test-UnderDest $dir)) { continue }
    try {
        $files = [System.IO.Directory]::GetFiles($dir)
        $subs  = [System.IO.Directory]::GetDirectories($dir)
    } catch { $unreadable++; continue }

    foreach ($f in $files) {
        if (Test-TransformerName ([System.IO.Path]::GetFileName($f)))    { Add-Job $f 'transformer' 'transformer' }
        elseif ($extSet.Contains([System.IO.Path]::GetExtension($f)))    { Add-Job $f 'cobol' 'sources' }
    }
    foreach ($s in $subs) {
        $name = [System.IO.Path]::GetFileName($s)
        if ($skipSet.Contains($name) -or (Test-ReparsePoint $s) -or (Test-UnderDest $s)) { continue }
        if (Test-TransformerName $name) { Add-TransformerTree $s } else { $stack.Push($s) }
    }
}

$nCobol = @($jobs | Where-Object Kind -eq 'cobol').Count
$nTrans = @($jobs | Where-Object Kind -eq 'transformer').Count
Write-Host ("Found {0} COBOL files and {1} transformer files. {2} unreadable folders skipped." -f $nCobol, $nTrans, $unreadable)

if ($DryRun) {
    $jobs | Sort-Object Kind, Source | Format-Table Kind, Source -AutoSize | Out-Host
    Write-Host "Dry run: nothing copied. Re-run without -DryRun to build $DestFull"
    return
}
if ($jobs.Count -eq 0) { Write-Warning 'Nothing found; no repo created.'; return }

# ---- 2. copy + verify ----------------------------------------------------------
New-Item -ItemType Directory -Path $DestFull -Force | Out-Null
$hashesSeen = @{}
$manifest   = [System.Collections.Generic.List[object]]::new()
$i = 0

foreach ($j in $jobs) {
    $i++
    Write-Progress -Activity 'Copying' -Status $j.Source -PercentComplete (100 * $i / $jobs.Count)
    $row = [ordered]@{ Kind = $j.Kind; SourcePath = $j.Source; RepoPath = ''; Bytes = 0; SHA256 = ''; LastWriteUtc = ''; Status = '' }
    try {
        $item = Get-Item -LiteralPath $j.Source -Force
        $hash = (Get-FileHash -LiteralPath $j.Source -Algorithm SHA256).Hash
        $row.Bytes        = $item.Length
        $row.SHA256       = $hash
        $row.LastWriteUtc = $item.LastWriteTimeUtc.ToString('o')

        if ($Dedupe -and $hashesSeen.ContainsKey($hash)) {
            $row.Status = 'duplicate-of:' + $hashesSeen[$hash]
        } else {
            New-Item -ItemType Directory -Path ([System.IO.Path]::GetDirectoryName($j.Target)) -Force | Out-Null
            Copy-Item -LiteralPath $j.Source -Destination $j.Target -Force
            $check = (Get-FileHash -LiteralPath $j.Target -Algorithm SHA256).Hash
            $rel   = $j.Target.Substring($DestFull.Length + 1)
            $row.RepoPath = $rel
            $row.Status   = $(if ($check -eq $hash) { 'copied-verified' } else { 'HASH-MISMATCH' })
            if (-not $hashesSeen.ContainsKey($hash)) { $hashesSeen[$hash] = $rel }
        }
    } catch {
        $row.Status = 'ERROR: ' + $_.Exception.Message
    }
    $manifest.Add([pscustomobject]$row)
}
Write-Progress -Activity 'Copying' -Completed

$manifest | Export-Csv -LiteralPath (Join-Path $DestFull 'MANIFEST.csv') -NoTypeInformation -Encoding UTF8

$ga = Join-Path $DestFull '.gitattributes'
if (-not (Test-Path -LiteralPath $ga)) {
    Set-Content -LiteralPath $ga -Encoding ASCII -Value @(
        '# Store every file byte-for-byte: COBOL is column-sensitive and may be EBCDIC.',
        '* -text'
    )
}

$ok  = @($manifest | Where-Object Status -eq 'copied-verified').Count
$dup = @($manifest | Where-Object { $_.Status -like 'duplicate-of:*' }).Count
$bad = $manifest.Count - $ok - $dup

# ---- 3. git --------------------------------------------------------------------
if (Get-Command git -ErrorAction SilentlyContinue) {
    Push-Location $DestFull
    try {
        if (-not (Test-Path -LiteralPath '.git')) {
            git init --quiet
            git config core.longpaths true
        }
        git add -A
        git commit --quiet -m "Collect COBOL sources and transformer projects ($ok files hash-verified)"
        if ($LASTEXITCODE -ne 0) {
            Write-Warning 'git commit did not complete (nothing new to commit, or git user.name / user.email not set).'
        }
    } finally { Pop-Location }
} else {
    Write-Warning 'git not found: files collected but not committed. Install Git, then in the folder run: git init; git add -A; git commit'
}

# ---- 4. report -----------------------------------------------------------------
Write-Host ''
Write-Host "Repo:        $DestFull"
Write-Host "Verified:    $ok"
Write-Host "Duplicates:  $dup"
if ($bad -gt 0) { Write-Warning "$bad files failed or mismatched - filter MANIFEST.csv on Status to see which." }
else            { Write-Host   'Failures:    0' }
