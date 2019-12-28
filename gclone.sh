#!/bin/bash

set -e

echo $0
#DEFINITIONS
name=$(basename "$0")
case $name in
glclone)
	provider="gitlab"
	;;
bbclone)
	provider="bitbucket"
	;;
ghclone)
	provider="github"
	;;
*)
	provider="all"
esac



home_dir="$HOME/.gclone"
cache_file="$home_dir/$provider-cache"
bitbucket_cache_file="$home_dir/bitbucket-cache"
gitlab_cache_file="$home_dir/gitlab-cache"
github_cache_file="$home_dir/github-cache"

crontab_backup_file="$home_dir/${provider}_crontab_backup"
tmp_crontab_file="$home_dir/${provider}_crontab_tmp"
log_file="$home_dir/$provider.log"

crontab_command="$0 --cache"
cron_interval_min="30"

#                       Minute               Hour   Day of Month       Month          Day of Week        Command
#                        (0-59)              (0-23)     (1-31)    (1-12 or Jan-Dec)  (0-6 or Sun-Sat)
crontab_entry="   */$cron_interval_min          *          *                *                *  . \$HOME/.profile; $crontab_command"


#FUCTIONS
function exit_error() {
	echo >&2 "$1"
	exit 1
}

function run_from_cache() {
	repos_url=$(fzf -m <"$cache_file")

	for repo in $repos_url; do
		repo_dir=$(echo "$repo" | awk -F'/' '{print $NF}'| sed 's/\.git//')

		git clone "$repo" --recursive
		cd "$repo_dir"
		git submodule foreach "git checkout master"
		cd ..
	done
}

function do_cache_urls (){
	if [ "$provider" = "all" ]
	then
		[ -f "$bitbucket_cache_file" ] || gclone-fetch bitbucket
		[ -f "$gitlab_cache_file" ] || gclone-fetch gitlab
		[ -f "$github_cache_file" ] || gclone-fetch github

		cat "$bitbucket_cache_file" > "$cache_file"
		cat "$gitlab_cache_file" >> "$cache_file"
		cat "$github_cache_file" >> "$cache_file"
	else
		gclone-fetch $provider
	fi
}

#CRON;;;;;;;;;;;;;;;;;;;;;
function is_in_crontab() {
	crontab -l | grep "$crontab_command" -q
}

function run() {
	if [ -f "$cache_file" ]; then
		run_from_cache
	else
		(do_cache_urls && run_from_cache) || exit_error "Couldn't download repos"
	fi
}

function install_crontab_entry() {
	crontab -l >"$crontab_backup_file"
	cp "$crontab_backup_file" "$tmp_crontab_file"
	echo "$crontab_entry" >>"$tmp_crontab_file"
	crontab "$tmp_crontab_file"
	rm "$tmp_crontab_file"
}

function remove_crontab_entry() {
	crontab -l >"$crontab_backup_file"
	cat "$crontab_backup_file" | grep -v "$crontab_command" >"$tmp_crontab_file"
	crontab "$tmp_crontab_file"
	rm "$tmp_crontab_file"
}

function print_help() {
	echo "${name}					# read urls from cache (or downloads if no cache yet) and runs ${name}"
	echo "${name} --cache				# download urls from $provider"
	echo "${name} --crontab-install		# installs cronjob that updates cache every $cron_interval_min minutes"
	echo "${name} --crontab-remove		# removes crontab entry"
	echo "${name} --force				# equivalent to '$name --cache && $name'"
}

function run_arg() {
	case $1 in
	"--cache")
		(do_cache_urls) || exit_error "Couldn't download repos"
		;;
	"--crontab-install")
		if is_in_crontab; then
			echo "Removing crontab entryfor ${name}"
			remove_crontab_entry
		fi
		echo "Installing crontab entry for ${name}"
		install_crontab_entry
		;;
	"--crontab-remove")
		if is_in_crontab; then
			echo "Removing crontab entry"
			remove_crontab_entry
		else
			echo "No ${name} entry in crontab"
		fi
		;;
	"--install")
		ln -sf "$0" ~/.local/bin/glclone
		ln -sf "$0" ~/.local/bin/bbclone
		ln -sf "$0" ~/.local/bin/ghclone
		;;
	"--force")
		echo "downloading repos"
		(do_cache_urls && run_from_cache) || exit_error "Couldn't download repos"
		;;
	*)
		print_help
		;;

	esac

}

#RUN
case $# in
0) run ;;
1) run_arg "$1" ;;
esac

touch "$log_file"
echo "$(date)| ${name} $*" >>"$log_file"
