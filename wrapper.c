#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include "wrapper.h"

void print_status(Story story)
{
	printf("Status:\n\t%s\n\tContinue?: %s\n\tChoice count: %i\n", 
			story == NULL ? "Ended" : "Unfinished", 
			CanContinue(story) ? "True":"False"	,
			ChoiceCount(story)
	);
}

int main() 
{
	printf("Initilizing:\n");
	void* story = Step(NULL); 
	do{
		print_status(story);

		printf("Stepping..\n");
		while(CanContinue(story))
		{
			story = Step(story);
			print_status(story);
		}

		unsigned int choice;
		printf("Chose choice: ");
		scanf("%u", &choice);
		ChooseChoiceIndex(story, choice);
		print_status(story);
		
		story = Step(story); 
	} while(ChoiceCount(story));

	return 0;
}
